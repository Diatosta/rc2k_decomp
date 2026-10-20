//! `physics::turn_by_car_matrix` (0x43F14E): the three floats at esi turned and placed by
//! `g_crashMatrix` (0x71A08C, the work copy's +0xB4; rows 0x10 apart, the place at +0x30)
//! with `scene::rotate_point`, and stored back (rounded to floats).
//!
//! In:  esi the point
//! Out: the point turned; every register kept
//!
//! `physics::turn_by_inverse` (0x43F118) is the same with the matrix at 0x71A9A4
//! (`g_carInverseMatrix`, made by `sub_43F16C` with `scene::invert_turn`).
//! Callers (6): the cars' physics under `sub_4454D2`.
//! Self-test: a copy of the original whose matrix is the test's (the car's is used all
//! through a frame; its call to `scene::rotate_point` goes to that hook), against the
//! Rust core on points and matrices by lot, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::math::x87_run::{DOUBLE, TEST_CONTROL_WORDS, with_control_word};
use crate::scene::matrix3::{extended, rounded, test_floats};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, close};
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::turn_by_car_matrix",
    rva(0x43F14E),
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

pub use crate::race::stage_prefs::CRASH_MATRIX_RVA; // g_crashMatrix

/// The point at `v` turned by the matrix at `m`, as the two functions do it: `fld z; fld
/// y; fld x`, `rotate_point`, `fstp x; fstp y; fstp z`.
pub unsafe fn turn_point_by(m: u32, v: u32) {
    unsafe {
        let p = v as *mut u32;
        let point = [
            extended(p.read_unaligned()),
            extended(p.add(1).read_unaligned()),
            extended(p.add(2).read_unaligned()),
        ];
        let out = rotate_point(m, point);
        p.write_unaligned(rounded(out[0]));
        p.add(1).write_unaligned(rounded(out[1]));
        p.add(2).write_unaligned(rounded(out[2]));
    }
}

/// The converted `turn_point_by` (`rc2k_core::vector`, plain floats): the hooks'.
pub unsafe fn turn_point_by_plain(m: u32, v: u32) {
    use crate::scene::vector3::{read_floats, write_floats};
    unsafe {
        let p: [f32; 3] = read_floats(v);
        let out = rc2k_core::vector::rotate_point(&read_floats(m), p.map(f64::from));
        write_floats(v, &out.map(|x| x as f32));
    }
}

/// The Rust version.
pub unsafe fn turn_by_car_matrix(v: u32) {
    unsafe { turn_point_by(game_addr(CRASH_MATRIX_RVA) as u32, v) }
}

/// The converted version.
pub unsafe fn turn_by_car_matrix_plain(v: u32) {
    unsafe { turn_point_by_plain(game_addr(CRASH_MATRIX_RVA) as u32, v) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { turn_by_car_matrix_plain(r.esi) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.esi, 12)], &rust);
}

/// The test of both: a copy of the original at `address` (0x1E bytes, its matrix
/// `variable` made the test's, its call at +0xF kept), against `turn_point_by`.
pub fn test_turn(hook: &RawHook, address: usize, entry: &[u8], variable: usize) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut matrix = vec![0u32; 16];
    let m = matrix.as_mut_ptr();
    let mut point = vec![0u32; 3];
    let v = point.as_mut_ptr();
    let copy = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len: 0x1E,
            entry,
            calls: &[0xF],
            jumps_out: &[],
        },
        &|_| None,
        &[(variable, m as u32)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let fill = |seed: u32| {
        let mut f = [0u32; 16];
        test_floats(seed * 7 + 3, &mut f);
        let mut q = [0u32; 3];
        test_floats(seed * 11 + 5, &mut q);
        for (n, x) in q.iter_mut().enumerate() {
            if (seed >> n) & 1 != 0 {
                *x ^= 0x8000_0000;
            }
        }
        unsafe {
            (m as *mut [u32; 16]).write(f);
            (v as *mut [u32; 3]).write(q);
        }
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=12u32 {
                fill(seed);
                let before = Regs {
                    esi: v as u32,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = unsafe { (v as *const [u32; 3]).read() };
                fill(seed);
                unsafe { turn_point_by_plain(m as u32, v as u32) };
                let mine = unsafe { (v as *const [u32; 3]).read() };
                let regs_ok = ro.preserved_except(&before, 0);
                // Exact at the game's double precision, within the tolerance at extended.
                let same = if word == DOUBLE {
                    theirs == mine
                } else {
                    theirs.iter().zip(&mine).all(|(&a, &b)| close(a, b, FLOAT))
                };
                t.check(regs_ok && same, || {
                    format!(
                        "control word {word:#x}, seed {seed}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        });
    }
    drop((matrix, point));
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_turn(
        &HOOK,
        0x43F14E,
        &[0xD9, 0x46, 0x08, 0xD9, 0x46, 0x04],
        0x71A08C,
    )
}
