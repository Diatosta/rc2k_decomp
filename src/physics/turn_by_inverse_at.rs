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
use crate::framework::Regs;
use crate::physics::turn_by_car_matrix::{turn_point_by, turn_point_by_plain};
use crate::screens::harness::Left;

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
