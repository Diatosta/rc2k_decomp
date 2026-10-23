//! `physics::turn_by_matrix_at` (0x45FC73): the three floats at esi turned and placed by
//! the matrix of the car block at edi (+0xB4; for the work car `g_crashMatrix`) with
//! `scene::rotate_point`, and stored back (rounded to floats). `physics::turn_by_car_matrix`
//! with the block given.
//!
//! In:  esi the point, edi the car block
//! Out: the point turned; every register kept
//!
//! Caller: `physics::apply_torque` (back into the world).
//! Self-test: `physics::turn_by_inverse_at`'s, on this function and offset.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::turn_by_car_matrix::{turn_point_by, turn_point_by_plain};
use crate::screens::harness::Left;

/// Where a car block's matrix is.
pub const MATRIX_OFFSET: u32 = 0xB4;

/// The Rust version.
pub unsafe fn turn_by_matrix_at(v: u32, car: u32) {
    unsafe { turn_point_by(car + MATRIX_OFFSET, v) }
}

/// The converted version (`turn_point_by_plain`): the hook's.
pub unsafe fn turn_by_matrix_at_plain(v: u32, car: u32) {
    unsafe { turn_point_by_plain(car + MATRIX_OFFSET, v) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { turn_by_matrix_at_plain(r.esi, r.edi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::floats::floats_entry(r, &rust)
}
