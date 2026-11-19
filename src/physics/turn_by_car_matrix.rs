//! Turns and places a point (three floats) by the work car's `crash_matrix` (the crash matrix; rows
//! 0x10 apart, the place at 0x30) with `scene::rotate_point`, and stores it back (rounded to
//! floats).
//!
//! `physics::turn_by_inverse` is the same with the car's `car_inverse_matrix` (made by
//! `race::invert_car_matrix` with `scene::invert_turn`).

use crate::math::cells::Cells;
use crate::math::matrix3::{extended, rounded_value};
use crate::physics::car_store::Car;
use crate::scene::rotate_point::rotate_point;

/// The point `v` turned by the matrix `m`: loaded as z, y, x, turned by `rotate_point` and stored
/// back as floats.
pub fn turn_cells(m: &(impl Cells + ?Sized), v: &(impl Cells + ?Sized)) {
    let point = std::array::from_fn(|k| extended(v.float(k)));
    let out = rotate_point(m, point);
    for (k, e) in out.iter().enumerate() {
        v.set_float(k, rounded_value(*e));
    }
}

pub fn turn_by_car_matrix(car: &Car, v: &(impl Cells + ?Sized)) {
    turn_cells(&car.crash_matrix, v);
}
