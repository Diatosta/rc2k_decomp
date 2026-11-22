//! Turns and places a point (three floats) by the work car's `car_inverse_matrix` (made from the
//! crash matrix by `race::invert_car_matrix`) with `scene::rotate_point`, and stores it back
//! (rounded to floats).
//!
//! See `physics/turn_by_car_matrix.rs` (the core and the test).

use crate::math::matrix3::{extended, rounded_value};
use crate::physics::car_store::Car;
use crate::scene::rotate_point::rotate_point;

/// The vector `v`, in floats, turned by the car's inverse matrix.
pub(super) fn turn_by_inverse(car: &Car, v: &mut [f32; 3]) {
    let out = rotate_point(&car.car_inverse_matrix, v.map(extended));
    *v = out.map(rounded_value);
}
