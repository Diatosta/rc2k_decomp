//! `physics::ease_pitch_down` (0x438CD3): half the way from `g_car` +0x6B8
//! (`unk_71A690`) down to `dword_608C1C` (unsigned halving, `shr`), kept in
//! `dword_608C24` and taken from +0x6B8. The twin of `physics::ease_pitch_up`.
//!
//! In:  nothing
//! Out: eax the step; the rest kept; no flags read
//!
//! Caller: `sub_43907F` (0x4392F3).

use crate::physics::car_store::CAR;
use crate::session::Cell32;
use rc2k_core::physics::easing::ease_down;

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.camera_pitch as &dyn Cell32,
        &crate::race::owner::race().physics.camera.pitch_floor,
        &crate::race::owner::race().physics.camera.pitch_step,
    ]
}

/// The Rust version on the variables' cells.
// Converted: `rc2k_core::physics::easing::ease_down`.
fn core(v: &[&dyn Cell32]) {
    let mut value = v[0].bits();
    let step = ease_down(&mut value, v[1].bits());
    v[2].set_bits(step);
    v[0].set_bits(value);
}

/// The Rust version.
pub fn ease_pitch_down() {
    core(&game());
}
