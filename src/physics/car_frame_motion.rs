//! Turns the work car's motion by its inverse matrix into the car's frame and stores it, rounded
//! to floats, as `local_velocity` (the third part is the car speed). The tail of
//! `physics::apply_force` on the work car.
//!
//! Caller: `physics::put_back_motion_vectors`.

use crate::math::cells::Cells;
use crate::scene::matrix3::{extended, rounded_value};
use crate::scene::rotate_point::rotate_point;

/// The motion `from` turned by the matrix `m` into `to`.
pub fn car_frame_motion(
    from: &(impl Cells + ?Sized),
    m: &(impl Cells + ?Sized),
    to: &(impl Cells + ?Sized),
) {
    let point = std::array::from_fn(|k| extended(from.float(k)));
    let out = rotate_point(m, point);
    for (k, e) in out.iter().enumerate() {
        to.set_float(k, rounded_value(*e));
    }
}
