//! Half the way from `camera_pitch` down to `pitch_floor` (unsigned halving), kept in
//! `pitch_step` and taken from the pitch. The twin of `physics::ease_pitch_up`.
//!
//! Returns the step.

use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use rc2k_core::physics::easing::ease_down;

/// On the variables' cells.
// Core: `rc2k_core::physics::easing::ease_down`.
pub fn ease_pitch_down(car: &Car, camera: &CameraWork) {
    let mut value = car.camera_pitch.get();
    let step = ease_down(&mut value, camera.pitch_floor.get());
    camera.pitch_step.set(step);
    car.camera_pitch.set(value);
}
