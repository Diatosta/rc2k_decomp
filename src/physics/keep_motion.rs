//! `physics::keep_motion` (0x440B7C): `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (0x71A184) into `dword_61409C`, `dword_6140A0`, `dword_6140A4` (eax pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B87E), every physics step.

use crate::race::owner_shared::Shared;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    for k in 0..3 {
        v[k + 3].set_bits(v[k].bits());
    }
}

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.local_velocity[0] as &dyn Cell32,
        &shared.car.local_velocity[1],
        &shared.car.local_velocity[2],
        &shared.physics_tables.kept_motion[0],
        &shared.physics_tables.kept_motion[1],
        &shared.physics_tables.kept_motion[2],
    ]
}

/// The Rust version.
pub fn keep_motion(shared: &Shared) {
    core(&game(shared));
}
