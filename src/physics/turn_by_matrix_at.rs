//! `physics::turn_by_matrix_at` (0x45FC73): the three floats at esi turned and placed by
//! the matrix of the car block at edi (+0xB4; for the work car `g_crashMatrix`) with
//! `scene::rotate_point`, and stored back (rounded to floats). `physics::turn_by_car_matrix`
//! with the block given.
//!
//! In:  esi the point, edi the car block
//! Out: the point turned; every register kept
//!
//! Caller: `physics::apply_torque` (back into the world).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::turn_by_car_matrix::turn_point_by;

/// Where a car block's matrix is.
pub const MATRIX_OFFSET: u32 = 0xB4;

/// The Rust version.
pub unsafe fn turn_by_matrix_at(v: u32, car: u32) {
    unsafe { turn_point_by(car + MATRIX_OFFSET, v) }
}
