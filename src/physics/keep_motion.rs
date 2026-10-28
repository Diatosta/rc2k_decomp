//! `physics::keep_motion` (0x440B7C): `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (0x71A184) into `dword_61409C`, `dword_6140A0`, `dword_6140A4` (eax pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B87E), every physics step.

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    for k in 0..3 {
        v[k + 3].set_bits(v[k].bits());
    }
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.local_velocity[0] as &dyn Cell32,
        &CAR.local_velocity[1],
        &CAR.local_velocity[2],
        &PHYSICS.kept_motion[0],
        &PHYSICS.kept_motion[1],
        &PHYSICS.kept_motion[2],
    ]
}

/// The Rust version.
pub fn keep_motion() {
    core(&game())
}
