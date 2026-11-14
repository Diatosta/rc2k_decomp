//! `camera_mode_is_2_or_3` = 1 when `camera_view` is 2 or 3, else 0. What the mode is is not known.
//!
//! Caller: `physics::feel_ground`, first.

use crate::physics::car_store::Car;
use rc2k_core::physics::ground::surface_flag;

// Core: `rc2k_core::physics::ground::surface_flag`.
pub fn note_surface_mode(car: &Car) {
    car.camera_mode_is_2_or_3
        .set(surface_flag(car.camera_view.get()));
}
