//! `physics::turn_by_inverse` (0x43F118): a point (three floats) turned and placed by
//! `g_carInverseMatrix` (the work car's `car_inverse_matrix`, made from `g_crashMatrix` by
//! `race::invert_car_matrix`) with `scene::rotate_point`, and stored back (rounded to floats).
//!
//! Takes: the point
//! Returns: the point turned
//!
//! See `physics/turn_by_car_matrix.rs` (the core and the test). Callers (13): the cars'
//! physics under `drive::race_step`.

use crate::math::cells::Cells;
use crate::physics::car_store::Car;
use crate::physics::turn_by_car_matrix::turn_cells;

pub fn turn_by_inverse(car: &Car, v: &(impl Cells + ?Sized)) {
    turn_cells(&car.car_inverse_matrix, v);
}
