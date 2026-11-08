//! `drive::pedal_inputs` (0x43A004): the work car's pedals from the input words. With the
//! keys' bit 0x4000 of `unk_71A440`: the throttle `unk_71A610` stepped by 0x10 toward 0x100
//! (`dword_613894` set) or 0, held there (by the sign of the steps, as the original); else
//! it is `dword_613894`. The same for the brake `unk_71A614` by bit 0x2000 and
//! `dword_613898`. The handbrake `unk_71A618` = `dword_61389C`, 0 when that is 0x100 or
//! less and `unk_71A400` is set.
//!
//! Returns: `dword_61389C`, `unk_71A440`
//!
//! Callers: `drive::car_controls`, `race::step_replay_cars`, `drive::replay_step`.

use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Dword;
use rc2k_core::drive::controls::{BRAKE_KEY, THROTTLE_KEY, handbrake, pedal};

/// One pedal: by the keys' `bit`, stepped from `input`, or set to it.
fn step_pedal(shared: &Shared, bit: u32, input: &Dword, at: &Dword) {
    at.set(pedal(
        shared.car.input_keys.get() & bit != 0,
        input.get(),
        at.get(),
    ));
}

/// The Rust version.
// Converted: `rc2k_core::drive::controls::{pedal, handbrake}`.
pub fn pedal_inputs(race: &Race) {
    step_pedal(
        &race.shared,
        THROTTLE_KEY,
        &race.shared.physics_tables.input_throttle,
        &race.shared.car.throttle,
    );
    step_pedal(
        &race.shared,
        BRAKE_KEY,
        &race.shared.physics_tables.input_brake,
        &race.shared.car.brake,
    );
    let hand = race.shared.physics_tables.input_handbrake.get();
    race.shared
        .car
        .handbrake
        .set(handbrake(hand, || race.shared.car.handbrake_failed.get()));
}
