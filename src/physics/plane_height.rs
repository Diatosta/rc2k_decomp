//! `physics::plane_height` (0x4425B6): `dword_639828` = c + (a - c) `flt_639810` + (b -
//! c) `flt_639814`, `fistp`'d (the control word's rounding), with a, b, c the whole
//! numbers `dword_639718`, `dword_639724`, `dword_63970C`: by the look of it the height
//! of a course face's plane under a point, from its corners' heights and the point's
//! shares (a reading; set by `sub_442006`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers (2): `sub_442006` (the face under a point, the ground's search) and
//! `sub_457B86`.
//! Self-test: a copy of the original on test variables, against the converted version,
//! values by lot (some large), exact at double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::copy_hooks_test::check_floats_at;
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{Floats, ROUNDED, dispatch_state_floats, floats_entry};
use rc2k_core::physics::ground::plane_height as plane_height_core;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::plane_height", rva(0x4425B6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables' addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct Plane {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub f: u32,
    pub g: u32,
    pub out: u32,
}

impl Plane {
    pub fn game() -> Self {
        let p = |x: usize| game_addr(rva(x)) as u32;
        Self {
            a: p(0x639718),
            b: p(0x639724),
            c: p(0x63970C),
            f: p(0x639810),
            g: p(0x639814),
            out: p(0x639828),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core on the variables at `p`.
pub unsafe fn plane_height_at(p: &Plane) {
    unsafe {
        let h = crate::math::x87_run::plane_height(
            at(p.a) as i32,
            at(p.b) as i32,
            at(p.c) as i32,
            at(p.f),
            at(p.g),
        );
        (p.out as *mut i32).write_unaligned(h);
    }
}

/// The converted version (`rc2k_core::physics::ground`, plain floats) on the variables
/// at `p`: the hook's.
// Converted: `rc2k_core::physics::ground::plane_height`.
pub unsafe fn plane_height_plain_at(p: &Plane) {
    unsafe {
        let h = plane_height_core(
            at(p.a) as i32,
            at(p.b) as i32,
            at(p.c) as i32,
            f32::from_bits(at(p.f)),
            f32::from_bits(at(p.g)),
        );
        (p.out as *mut i32).write_unaligned(h);
    }
}

/// The Rust version (the exact one, for `physics::ground`, `face_under_point` and the
/// race's shadows).
pub unsafe fn plane_height() {
    unsafe { plane_height_at(&Plane::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { plane_height_plain_at(&Plane::game()) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let out = Plane::game().out;
    let floats = Floats {
        memory: &[((out, 4), ROUNDED)],
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[(out, 4)], &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 6];
    let base = cells.as_mut_ptr() as u32;
    let p = Plane {
        a: base,
        b: base + 4,
        c: base + 8,
        f: base + 12,
        g: base + 16,
        out: base + 20,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4425B6,
            len: 0x39,
            entry: &[0xDB, 0x05, 0x18, 0x97, 0x63, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x639718, p.a),
            (0x639724, p.b),
            (0x63970C, p.c),
            (0x639810, p.f),
            (0x639814, p.g),
            (0x639828, p.out),
        ],
        8,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let fill = |seed: u32| {
        let mut f = [0u32; 2];
        test_floats(seed * 13 + 7, &mut f);
        let mut s = seed.wrapping_mul(0x9E37_79B9);
        let mut next = || {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            // Course heights are large fixed-point numbers; some cases near the limits.
            if seed.is_multiple_of(5) {
                s
            } else {
                (s as i32 >> 8) as u32
            }
        };
        let (a, b, c) = (next(), next(), next());
        unsafe {
            for (k, v) in [a, b, c, f[0] & 0x3FFF_FFFF, f[1], 0xAAAA_AAAA]
                .into_iter()
                .enumerate()
            {
                ((base + 4 * k as u32) as *mut u32).write_volatile(v);
            }
        }
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=20u32 {
                fill(seed);
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = unsafe { (base as *const [u32; 6]).read() };
                fill(seed);
                unsafe { plane_height_plain_at(&p) };
                let mine = unsafe { (base as *const [u32; 6]).read() };
                let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                let bytes =
                    |w: [u32; 6]| w.iter().flat_map(|d| d.to_le_bytes()).collect::<Vec<u8>>();
                check_floats_at(
                    &mut t,
                    "physics::plane_height",
                    word,
                    &[(base, 24)],
                    &bytes(theirs),
                    &bytes(mine),
                    &[((p.out, 4), ROUNDED)],
                    regs_ok,
                    || {
                        format!(
                            "control word {word:#x}, seed {seed}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                        )
                    },
                );
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
