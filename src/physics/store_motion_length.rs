//! `physics::store_motion_length` (0x43BD2D): `speed` = the length of the three
//! floats at `velocity` (`physics::vector_length`, then `fstp`: rounded to a float).
//! Both are the work copy's (+0x198, +0x1C4); that the vector is the car's motion is a
//! reading (it sits next to `g_carSpeed`), not seen.
//!
//! Callers (2): `physics::car_physics_step` (0x43B644, 0x43B94D), the car's physics step under
//! `drive::race_step`.

use crate::physics::car_store::Car;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;

/// The exact version, for `physics::car_physics_step` and `physics::keep_motion_vectors`: the
/// length of the car's motion.
pub fn store_motion_length(car: &Car) {
    car.speed.set_bits(rounded(vector_length(&car.velocity)));
}
