//! `physics::ease_pitch_down` (0x438CD3): half the way from `g_car` +0x6B8
//! (`unk_71A690`) down to `dword_608C1C` (unsigned halving, `shr`), kept in
//! `dword_608C24` and taken from +0x6B8. The twin of `physics::ease_pitch_up`.
//!
//! Returns: the step
//!
//! Caller: `physics::camera_avoid_ground` (0x4392F3).

use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::session::Cell32;
use rc2k_core::physics::easing::ease_down;

fn game<'a>(car: &'a Car, camera: &'a CameraWork) -> Vec<&'a dyn Cell32> {
    vec![
        &car.camera_pitch as &dyn Cell32,
        &camera.pitch_floor,
        &camera.pitch_step,
    ]
}

/// On the variables' cells.
// Converted: `rc2k_core::physics::easing::ease_down`.
fn core(v: &[&dyn Cell32]) {
    let mut value = v[0].bits();
    let step = ease_down(&mut value, v[1].bits());
    v[2].set_bits(step);
    v[0].set_bits(value);
}

pub fn ease_pitch_down(car: &Car, camera: &CameraWork) {
    core(&game(car, camera));
}
