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

use crate::physics::turn_by_car_matrix::turn_point_by;

/// Where a car block's inverse matrix is.
pub const INVERSE_OFFSET: u32 = 0x9CC;

/// The Rust version.
pub unsafe fn turn_by_inverse_at(v: u32, car: u32) {
    unsafe { turn_point_by(car + INVERSE_OFFSET, v) }
}
