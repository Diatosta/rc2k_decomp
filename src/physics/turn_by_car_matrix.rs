//! `physics::turn_by_car_matrix` (0x43F14E): a point (three floats) turned and placed by
//! `g_crashMatrix` (the work car's `crash_matrix`; rows 0x10 apart, the place at +0x30) with
//! `scene::rotate_point`, and stored back (rounded to floats).
//!
//! Takes: the point
//! Returns: the point turned
//!
//! `physics::turn_by_inverse` is the same with the car's `car_inverse_matrix` (`g_carInverseMatrix`,
//! made by `race::invert_car_matrix` with `scene::invert_turn`).
//! Callers (6): the cars' physics under `drive::race_step`.

use crate::math::cells::Cells;
use crate::physics::car_store::Car;
use crate::scene::matrix3::{extended, rounded_value};
use crate::scene::rotate_point::rotate_point;

// g_crashMatrix

/// The point `v` turned by the matrix `m`, as the two functions do it: `fld z; fld y; fld
/// x`, `rotate_point`, `fstp x; fstp y; fstp z`.
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
