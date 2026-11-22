//! Turns the work car's motion by its inverse matrix into the car's frame and stores it, rounded
//! to floats, as `local_velocity` (the third part is the car speed). The tail of
//! `physics::apply_force` on the work car.

use crate::math::matrix3::{extended, rounded_value};
use crate::scene::rotate_point::rotate_point;

/// The motion `from` turned by the matrix `m` into `to`.
pub(super) fn car_frame_motion(from: &[f32; 3], m: &[f32; 16], to: &mut [f32; 3]) {
    let point = from.map(extended);
    let out = rotate_point(m, point);
    for (k, e) in out.iter().enumerate() {
        to[k] = rounded_value(*e);
    }
}
