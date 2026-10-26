//! `physics::ease_pitch_up` (0x438CBA): half the way from `g_car` +0x6B8 (`unk_71A690`)
//! up to `dword_608C20` (unsigned halving, `shr`), kept in `dword_608C24` and added to
//! +0x6B8. `physics::ease_pitch_down` (0x438CD3) is its twin towards `dword_608C1C`.
//!
//! In:  nothing
//! Out: eax the step; the rest kept; no flags read (the caller compares `dword_608C24`)
//!
//! Caller: `sub_43907F` (0x4392CD).

use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put};
use rc2k_core::physics::easing::ease_up;

fn game() -> Vec<u32> {
    vec![
        CAMERA_WORK.pitch_ceiling.addr() as u32,
        CAR.camera_pitch.addr() as u32,
        CAMERA_WORK.pitch_step.addr() as u32,
    ]
}

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::easing::ease_up`.
fn core(v: &[u32]) {
    let mut value = get(v[1]);
    let step = ease_up(&mut value, get(v[0]));
    put(v[2], step);
    put(v[1], value);
}

/// The Rust version.
pub fn ease_pitch_up() {
    core(&game())
}
