//! `physics::car_frame_motion` (0x43DB71): the work car's motion (`velocity` ..,
//! `g_car` +0x198) turned by `g_carInverseMatrix` (`scene::rotate_point`) into the car's
//! frame, stored (rounded to floats) at `local_velocity`, `dword_71A180`, `g_carSpeed`
//! (+0x1A4 .. +0x1AC). The tail of `physics::apply_force` on the work car.
//!
//! Caller: `physics::put_back_motion_vectors` (its tail).

use crate::math::cells::Cells;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The motion `from` turned by the matrix `m` into `to`.
pub fn car_frame_motion(
    from: &(impl Cells + ?Sized),
    m: &(impl Cells + ?Sized),
    to: &(impl Cells + ?Sized),
) {
    let point = std::array::from_fn(|k| extended(from.bits(k)));
    let out = rotate_point(m, point);
    for (k, e) in out.iter().enumerate() {
        to.set_bits(k, rounded(*e));
    }
}
