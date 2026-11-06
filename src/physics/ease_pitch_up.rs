//! `physics::ease_pitch_up` (0x438CBA): half the way from `g_car` +0x6B8 (`unk_71A690`)
//! up to `dword_608C20` (unsigned halving, `shr`), kept in `dword_608C24` and added to
//! +0x6B8. `physics::ease_pitch_down` (0x438CD3) is its twin towards `dword_608C1C`.
//!
//! In:  nothing
//! Out: eax the step; the rest kept; no flags read (the caller compares `dword_608C24`)
//!
//! Caller: `sub_43907F` (0x4392CD).

use crate::physics::car_store::CAR;
use crate::race::owner::race;
use crate::session::Cell32;
use rc2k_core::physics::easing::ease_up;

fn game() -> Vec<&'static dyn Cell32> {
    let physics = &race().physics;
    vec![
        &physics.camera.pitch_ceiling as &dyn Cell32,
        &CAR.camera_pitch,
        &physics.camera.pitch_step,
    ]
}

/// The Rust version on the variables' cells.
// Converted: `rc2k_core::physics::easing::ease_up`.
fn core(v: &[&dyn Cell32]) {
    let mut value = v[1].bits();
    let step = ease_up(&mut value, v[0].bits());
    v[2].set_bits(step);
    v[1].set_bits(value);
}

/// The Rust version.
pub fn ease_pitch_up() {
    core(&game());
}
