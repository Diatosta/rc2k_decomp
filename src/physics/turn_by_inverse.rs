//! Turns and places a point (three floats) by the work car's `car_inverse_matrix` (made from the
//! crash matrix by `race::invert_car_matrix`) with `scene::rotate_point`, and stores it back
//! (rounded to floats).
//!
//! See `physics/turn_by_car_matrix.rs` (the core and the test). Callers: the cars' physics under
//! `drive::race_step`, 13 calls.

use crate::math::cells::Cells;
use crate::physics::car_store::Car;
use crate::physics::turn_by_car_matrix::turn_cells;

pub fn turn_by_inverse(car: &Car, v: &(impl Cells + ?Sized)) {
    turn_cells(&car.car_inverse_matrix, v);
}
