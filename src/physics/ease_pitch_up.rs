//! `physics::ease_pitch_up` (0x438CBA): half the way from `g_car` +0x6B8 (`camera_pitch`)
//! up to `pitch_ceiling` (unsigned halving, `shr`), kept in `pitch_step` and added to
//! +0x6B8. `physics::ease_pitch_down` (0x438CD3) is its twin towards `pitch_floor`.
//!
//! Returns: the step
//!
//! Caller: `physics::camera_avoid_ground` (0x4392CD).

use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::session::Cell32;
use rc2k_core::physics::easing::ease_up;

fn game<'a>(car: &'a Car, camera: &'a CameraWork) -> Vec<&'a dyn Cell32> {
    vec![
        &camera.pitch_ceiling as &dyn Cell32,
        &car.camera_pitch,
        &camera.pitch_step,
    ]
}

/// On the variables' cells.
// Converted: `rc2k_core::physics::easing::ease_up`.
fn core(v: &[&dyn Cell32]) {
    let mut value = v[1].bits();
    let step = ease_up(&mut value, v[0].bits());
    v[2].set_bits(step);
    v[1].set_bits(value);
}

pub fn ease_pitch_up(car: &Car, camera: &CameraWork) {
    core(&game(car, camera));
}
