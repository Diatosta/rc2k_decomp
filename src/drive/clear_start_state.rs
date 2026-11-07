//! `drive::clear_start_state` (0x4376E3): six of the work copy's start values 0:
//! `dword_71A234` (the grades' mode, `physics::grade_or_reset`), `dword_71A070`,
//! `dword_71AA88`, `flt_71A650`, `flt_71A658`, `dword_71A01C`.
//!
//! Caller: `sub_43703E` (0x4371BC, 0x43729B).

use crate::race::owner_shared::Shared;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    for cell in v {
        cell.set_bits(0);
    }
}

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.driver_kind,
        &shared.car.checkpoint,
        &shared.car.engine_sound_speed_int,
        &shared.car.start_share,
        &shared.car.start_blend,
        &shared.car.unknown_44,
    ]
}

/// The Rust version.
pub fn clear_start_state(shared: &Shared) {
    core(&game(shared));
}
