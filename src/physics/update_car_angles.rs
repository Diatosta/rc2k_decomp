//! `physics::car_angles` when `camera_mode_is_2_or_3` is set or `all_wheels_off` is 0; otherwise
//! nothing.
//!
//! Caller: `physics::car_physics_step`, twice, after `race::invert_car_matrix`.

use crate::physics::car_angles::car_angles;
use crate::physics::car_store::Car;
use crate::race::owner_course::Course;

use rc2k_core::physics::motion::angles_wanted as wanted;

// Core: `rc2k_core::physics::motion::angles_wanted`.
pub fn update_car_angles(course: &Course, car: &Car, scene: &crate::scene::owner::Scene) {
    let (force, hold) = (car.camera_mode_is_2_or_3.get(), car.all_wheels_off.get());
    if wanted(force, hold) {
        car_angles(course, car, scene);
    }
}
