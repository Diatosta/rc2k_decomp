//! `drive::clear_start_state` (0x4376E3): six of the work copy's start values 0: `driver_kind`
//! (the grades' mode, `physics::grade_or_reset`), `checkpoint`, `engine_sound_speed_int`, `start_share`,
//! `start_blend`, `dword_71A01C`.
//!
//! Caller: `drive::start_cars_behind` (0x4371BC, 0x43729B).

use crate::race::owner_shared::RaceShared;
use crate::session::Cell32;

fn core(v: &[&dyn Cell32]) {
    for cell in v {
        cell.set_bits(0);
    }
}

fn game(shared: &RaceShared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.driver_kind,
        &shared.car.checkpoint,
        &shared.car.engine_sound_speed_int,
        &shared.car.start_share,
        &shared.car.start_blend,
        &shared.car.unused_44,
    ]
}

pub fn clear_start_state(shared: &RaceShared) {
    core(&game(shared));
}
