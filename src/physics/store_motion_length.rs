//! `speed` = the length of the three floats at `velocity` (`physics::vector_length`, rounded to a
//! float). Both are the work copy's; that the vector is the car's motion is a reading (it sits next
//! to the car speed), not seen.

use crate::math::matrix3::rounded_value;
use crate::physics::car_store::Car;
use crate::physics::vector_length::vector_length;

/// The exact version, for `physics::car_physics_step` and `physics::keep_motion_vectors`: the
/// length of the car's motion.
pub(super) fn store_motion_length(car: &Car) {
    car.speed.set(rounded_value(vector_length(&car.velocity)));
}
