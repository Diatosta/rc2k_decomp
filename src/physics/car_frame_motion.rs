//! `physics::car_frame_motion` (0x43DB71): the work car's motion (`dword_71A170` ..,
//! `g_car` +0x198) turned by `g_carInverseMatrix` (`scene::rotate_point`) into the car's
//! frame, stored (rounded to floats) at `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (+0x1A4 .. +0x1AC). The tail of `physics::apply_force` on the work car.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43BD8B` (its tail).
//! Self-test: a copy of the original (the motion, the result and the matrix the test's,
//! its call kept) against the Rust core, on values by lot under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::turn_by_inverse::INVERSE_RVA;
use crate::scene::matrix3::{extended, rounded, test_floats};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::car_frame_motion", rva(0x43DB71), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MOTION_RVA: usize = rva(0x71A170); // dword_71A170, g_car + 0x198
const LOCAL_RVA: usize = rva(0x71A17C); // dword_71A17C, g_car + 0x1A4

/// The Rust core: the motion at `from` turned by the matrix at `m` into `to`.
pub unsafe fn car_frame_motion_at(from: u32, m: u32, to: u32) {
    unsafe {
        let p = from as *const u32;
        let point = [
            extended(p.read_unaligned()),
            extended(p.add(1).read_unaligned()),
            extended(p.add(2).read_unaligned()),
        ];
        let out = rotate_point(m, point);
        let q = to as *mut u32;
        for (k, e) in out.iter().enumerate() {
            q.add(k).write_unaligned(rounded(*e));
        }
    }
}

/// The Rust version.
pub unsafe fn car_frame_motion() {
    unsafe {
        car_frame_motion_at(
            game_addr(MOTION_RVA) as u32,
            game_addr(INVERSE_RVA) as u32,
            game_addr(LOCAL_RVA) as u32,
        )
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { car_frame_motion() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(game_addr(LOCAL_RVA) as u32, 12)],
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// A copy of the original whose motion, result and matrix are at `from`, `to`, `m`.
pub fn test_copy(from: u32, to: u32, m: u32) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43DB71,
            len: 0x32,
            entry: &[0xD9, 0x05, 0x78, 0xA1, 0x71, 0x00],
            calls: &[0x19],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x71A170, from),
            (0x71A174, from + 4),
            (0x71A178, from + 8),
            (0x71A17C, to),
            (0x71A180, to + 4),
            (0x71A184, to + 8),
            (0x71A9A4, m),
        ],
        7,
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The matrix (16), the motion (3), the result (3).
    let mut cells = vec![0u32; 22];
    let m = cells.as_mut_ptr() as u32;
    let from = m + 0x40;
    let to = from + 12;
    let copy = match test_copy(from, to, m) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let fill = |seed: u32| {
        let mut f = [0u32; 22];
        test_floats(seed * 13 + 1, &mut f);
        for (k, x) in f.iter_mut().enumerate() {
            if k >= 16 && (seed >> (k - 16)) & 1 != 0 {
                *x ^= 0x8000_0000;
            }
        }
        unsafe { (m as *mut [u32; 22]).write(f) };
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=12u32 {
                fill(seed);
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = unsafe { (m as *const [u32; 22]).read() };
                fill(seed);
                unsafe { car_frame_motion_at(from, m, to) };
                let mine = unsafe { (m as *const [u32; 22]).read() };
                let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                t.check(regs_ok && theirs == mine, || {
                    format!(
                        "control word {word:#x}, seed {seed}: regs_ok={regs_ok} original {:x?}; rust {:x?}",
                        &theirs[19..],
                        &mine[19..]
                    )
                });
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
