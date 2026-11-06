//! `drive::clear_start_state` (0x4376E3): six of the work copy's start values 0:
//! `dword_71A234` (the grades' mode, `physics::grade_or_reset`), `dword_71A070`,
//! `dword_71AA88`, `flt_71A650`, `flt_71A658`, `dword_71A01C`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::store_car` next)
//!
//! Caller: `sub_43703E` (0x4371BC, 0x43729B).

use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    for cell in v {
        cell.set_bits(0);
    }
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &crate::race::owner::race().shared.car.driver_kind,
        &crate::race::owner::race().shared.car.checkpoint,
        &crate::race::owner::race().shared.car.engine_sound_speed_int,
        &crate::race::owner::race().shared.car.start_share,
        &crate::race::owner::race().shared.car.start_blend,
        &crate::race::owner::race().shared.car.unknown_44,
    ]
}

/// The Rust version.
pub fn clear_start_state() {
    core(&game());
}
