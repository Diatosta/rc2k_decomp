//! `physics::turn_by_inverse` (0x43F118): the three floats at esi turned and placed by
//! `g_carInverseMatrix` (0x71A9A4, the work copy's +0x9CC, made from `g_crashMatrix` by
//! `race::invert_car_matrix`) with `scene::rotate_point`, and stored back (rounded to floats).
//!
//! Takes: the point
//! Returns: the point turned
//!
//! See `physics/turn_by_car_matrix.rs` (the core and the test). Callers (13): the cars'
//! physics under `drive::race_step`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::cells::Cells;
use crate::physics::turn_by_car_matrix::turn_cells;
use crate::race::owner_shared::RaceShared;

pub fn turn_by_inverse(shared: &RaceShared, v: &(impl Cells + ?Sized)) {
    turn_cells(&shared.car.car_inverse_matrix, v);
}
