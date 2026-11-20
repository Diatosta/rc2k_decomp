//! Half the way from `camera_pitch` up to `pitch_ceiling` (signed halving), kept in
//! `pitch_step` and added to the pitch. `physics::ease_pitch_down` is its twin towards
//! `pitch_floor`.
//!
//! Returns the step.

use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use rc2k_core::physics::easing::ease_up;

/// On the variables' cells.
// Core: `rc2k_core::physics::easing::ease_up`.
pub(super) fn ease_pitch_up(car: &Car, camera: &mut CameraWork) {
    let mut value = car.camera_pitch.get();
    let step = ease_up(&mut value, camera.pitch_ceiling);
    camera.pitch_step = step;
    car.camera_pitch.set(value);
}
