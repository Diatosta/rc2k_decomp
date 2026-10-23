//! `physics::turn_by_inverse_at` (0x45FC55): the three floats at esi turned and placed by
//! the inverse matrix of the car block at edi (+0x9CC; for the work car
//! `g_carInverseMatrix`) with `scene::rotate_point`, and stored back (rounded to floats).
//! `physics::turn_by_inverse` with the block given.
//!
//! In:  esi the point, edi the car block
//! Out: the point turned; every register kept
//!
//! `physics::turn_by_matrix_at` (0x45FC73) is the same with the block's +0xB4 (for the
//! work car `g_crashMatrix`).
//! Caller: `physics::apply_torque` (into the car's frame).
//! Self-test: the original on a test block (matrix by lot) and point by lot, against
//! the Rust core, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest};
use crate::math::x87_run::{DOUBLE, TEST_CONTROL_WORDS, with_control_word};
use crate::physics::turn_by_car_matrix::{turn_point_by, turn_point_by_plain};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, close};

/// Where a car block's inverse matrix is.
pub const INVERSE_OFFSET: u32 = 0x9CC;

/// The Rust version.
pub unsafe fn turn_by_inverse_at(v: u32, car: u32) {
    unsafe { turn_point_by(car + INVERSE_OFFSET, v) }
}

/// The converted version (`turn_point_by_plain`): the hook's.
pub unsafe fn turn_by_inverse_at_plain(v: u32, car: u32) {
    unsafe { turn_point_by_plain(car + INVERSE_OFFSET, v) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { turn_by_inverse_at_plain(r.esi, r.edi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::floats::floats_entry(r, &rust)
}

/// The test of both: `original` (the hook's `call_original`) on a test block whose
/// matrix at `offset` is by lot, against `turn_point_by`.
pub fn test_turn_at(
    hook: &RawHook,
    original: unsafe extern "C" fn(*mut Regs),
    offset: u32,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut block = vec![0u32; 0xA20 / 4];
    let car = block.as_mut_ptr() as u32;
    let mut point = vec![0u32; 3];
    let v = point.as_mut_ptr();
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
            ((car + offset) as *mut [u32; 16]).write_unaligned(f);
            (v as *mut [u32; 3]).write(q);
        }
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=12u32 {
                fill(seed);
                let before = Regs {
                    esi: v as u32,
                    edi: car,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { original(&mut ro) };
                let theirs = unsafe { (v as *const [u32; 3]).read() };
                fill(seed);
                unsafe { turn_point_by_plain(car + offset, v as u32) };
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
    drop((block, point));
    t.finish()
}
