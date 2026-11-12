//! `physics::ease_pitch_up` (0x438CBA): half the way from `g_car` +0x6B8 (`camera_pitch`)
//! up to `pitch_ceiling` (unsigned halving, `shr`), kept in `pitch_step` and added to
//! +0x6B8. `physics::ease_pitch_down` (0x438CD3) is its twin towards `pitch_floor`.
//!
//! Returns: the step
//!
//! Caller: `physics::camera_avoid_ground` (0x4392CD).

use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use rc2k_core::physics::easing::ease_up;

/// On the variables' cells.
// Converted: `rc2k_core::physics::easing::ease_up`.
pub fn ease_pitch_up(car: &Car, camera: &CameraWork) {
    let mut value = car.camera_pitch.get();
    let step = ease_up(&mut value, camera.pitch_ceiling.get());
    camera.pitch_step.set(step);
    car.camera_pitch.set(value);
}
