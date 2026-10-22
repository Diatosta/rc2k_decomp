//! `physics::put_back_motion_vectors` (0x43BD8B): the three dwords at `dword_614BCC` ..
//! back into the work car's motion (`dword_71A170` .., `g_car` +0x198) and those at
//! `dword_614BDC` .. into its spin (`dword_71A15C` .., +0x184), eax pushed around; then
//! `physics::car_frame_motion`. The undoing of `physics::keep_motion_vectors`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B8F1, before `sub_43F7AC`).
//! Self-test: a copy of the original (all its variables the test's, its call to
//! `car_frame_motion`'s test copy on the same) against the Rust core, values by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::car_frame_motion::{car_frame_motion_at, test_copy};
use crate::physics::turn_by_inverse::INVERSE_RVA;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::put_back_motion_vectors",
    rva(0x43BD8B),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Where the vectors are: the game's or a test's.
#[derive(Clone, Copy)]
pub struct MotionPlaces {
    pub kept_motion: u32,
    pub kept_spin: u32,
    pub motion: u32,
    pub spin: u32,
    pub local: u32,
    pub inverse: u32,
}

impl MotionPlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            kept_motion: g(0x614BCC),
            kept_spin: g(0x614BDC),
            motion: g(0x71A170),
            spin: g(0x71A15C),
            local: g(0x71A17C),
            inverse: game_addr(INVERSE_RVA) as u32,
        }
    }

    pub fn regions(&self) -> Vec<Region> {
        vec![(self.motion, 12), (self.spin, 12), (self.local, 12)]
    }
}

/// The Rust core.
pub unsafe fn put_back_motion_vectors_at(p: &MotionPlaces) {
    unsafe {
        for k in 0..3usize {
            let m = (p.kept_motion as *const u32).add(k).read_unaligned();
            (p.motion as *mut u32).add(k).write_unaligned(m);
        }
        for k in 0..3usize {
            let s = (p.kept_spin as *const u32).add(k).read_unaligned();
            (p.spin as *mut u32).add(k).write_unaligned(s);
        }
        car_frame_motion_at(p.motion, p.inverse, p.local);
    }
}

/// The Rust version.
pub unsafe fn put_back_motion_vectors() {
    unsafe { put_back_motion_vectors_at(&MotionPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { put_back_motion_vectors() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &MotionPlaces::game().regions(),
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The matrix (16), then kept motion, kept spin, motion, spin, local (3 each).
    let mut cells = vec![0u32; 16 + 15];
    let base = cells.as_mut_ptr() as u32;
    let p = MotionPlaces {
        inverse: base,
        kept_motion: base + 0x40,
        kept_spin: base + 0x4C,
        motion: base + 0x58,
        spin: base + 0x64,
        local: base + 0x70,
    };
    let inner = match test_copy(p.motion, p.local, p.inverse) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let mut vars = vec![];
    for k in 0..3u32 {
        vars.push((0x614BCC + 4 * k as usize, p.kept_motion + 4 * k));
        vars.push((0x71A170 + 4 * k as usize, p.motion + 4 * k));
        vars.push((0x614BDC + 4 * k as usize, p.kept_spin + 4 * k));
        vars.push((0x71A15C + 4 * k as usize, p.spin + 4 * k));
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43BD8B,
            len: 0x46,
            entry: &[0x50, 0xA1, 0xCC, 0x4B, 0x61, 0x00],
            calls: &[0x40],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43DB71)).then_some(target),
        &vars,
        12,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    let fill = |seed: u32| {
        let mut f = [0u32; 31];
        test_floats(seed * 17 + 9, &mut f);
        unsafe { (base as *mut [u32; 31]).write(f) };
    };
    for seed in 1..=8u32 {
        fill(seed);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill(seed);
        unsafe { put_back_motion_vectors_at(&p) };
        let mine = capture(&all);
        let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
        t.check(regs_ok && theirs == mine, || {
            format!("seed {seed}: regs_ok={regs_ok}; the memory differs")
        });
    }
    drop((copy, inner, cells));
    t.finish()
}
