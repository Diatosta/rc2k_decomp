//! `drive::pedal_inputs` (0x43A004): the work car's pedals from the input words. With the keys' bit
//! 0x4000 of `input_keys`: the throttle `throttle` stepped by 0x10 toward 0x100 (`input_throttle`
//! set) or 0, held there (by the sign of the steps, as the original); else it is `input_throttle`.
//! The same for the brake `brake` by bit 0x2000 and `input_brake`. The handbrake `handbrake`
//! = `input_handbrake`, 0 when that is 0x100 or less and `handbrake_failed` is set.
//!
//! Returns: `input_handbrake`, `input_keys`
//!
//! Callers: `drive::car_controls`, `race::step_replay_cars`, `drive::replay_step`.

use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::session::Dword;
use rc2k_core::drive::controls::{BRAKE_KEY, THROTTLE_KEY, handbrake, pedal};

/// One pedal: by the keys' `bit`, stepped from `input`, or set to it.
fn step_pedal(shared: &RaceShared, bit: u32, input: &Dword, at: &Dword) {
    at.set(pedal(
        shared.car().input_keys.get() & bit != 0,
        input.get(),
        at.get(),
    ));
}

// Converted: `rc2k_core::drive::controls::{pedal, handbrake}`.
pub fn pedal_inputs(race: &Race) {
    step_pedal(
        &race.shared,
        THROTTLE_KEY,
        &race.shared.physics_tables.input_throttle,
        &race.shared.car().throttle,
    );
    step_pedal(
        &race.shared,
        BRAKE_KEY,
        &race.shared.physics_tables.input_brake,
        &race.shared.car().brake,
    );
    let hand = race.shared.physics_tables.input_handbrake.get();
    race.shared
        .car()
        .handbrake
        .set(handbrake(hand, || race.shared.car().handbrake_failed.get()));
}
