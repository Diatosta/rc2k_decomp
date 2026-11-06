//! `physics::gear_input_rules` (0x439BF6): the gearbox's input state (0x613890 ..
//! 0x6138A8) by the car's state (the work copy's `dword_71A244`, `dword_71A010`,
//! `dword_719FEC`, `dword_71A07C`, `dword_71A078`, `dword_71A080`): with `dword_71A244`
//! neither 0 nor -1 all cleared, 0x61389C = 500 and `g_carGear` 0 (nothing else); with
//! `dword_71A010` 2 the two inputs 0x613894 / 0x613898 joined into the first (in neutral,
//! `dword_6138A8` = 1, or -1 when the second is the larger) or swapped (in reverse);
//! with `dword_719FEC` below 800 (the sign of the difference) 0x61389C = 500 and, unless
//! `dword_62389C` is 1, four cleared; each of three switches clears them and the gear.
//! What they are is a reading at best (an automatic box's held inputs?).
//!
//! In:  nothing
//! Out: eax the joined input when they were joined; the rest kept; no flags read
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F` (each calls
//! `sub_43B635` or `sub_439DB1` next).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::race::owner::race;
use crate::session::Cell32;
use rc2k_core::physics::gearbox::{GearCar, GearInputs};

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// The Rust version on the variables' cells `v`; what the rules change is written back. The
/// eax it leaves (the joined input) is read by none of its callers.
// Converted: `rc2k_core::physics::gearbox::GearInputs::apply_rules`.
pub fn rules_at(v: &[&dyn Cell32]) {
    let car = GearCar {
        state: get(v, 0),
        box_mode: get(v, 7),
        timer: get(v, 9),
        race_mode: get(v, 10),
        switches: [get(v, 12), get(v, 13), get(v, 14)],
    };
    let before = GearInputs {
        w90: get(v, 11),
        first: get(v, 3),
        second: get(v, 4),
        hold: get(v, 5),
        wa0: get(v, 2),
        wa4: get(v, 1),
        joined: get(v, 8),
    };
    let gear_before = get(v, 6);
    let (mut inputs, mut gear) = (before, gear_before);
    inputs.apply_rules(&mut gear, &car);
    for (k, now, was) in [
        (11, inputs.w90, before.w90),
        (3, inputs.first, before.first),
        (4, inputs.second, before.second),
        (5, inputs.hold, before.hold),
        (2, inputs.wa0, before.wa0),
        (1, inputs.wa4, before.wa4),
        (8, inputs.joined, before.joined),
        (6, gear, gear_before),
    ] {
        if now != was {
            put(v, k, now);
        }
    }
}

fn game() -> Vec<&'static dyn Cell32> {
    let race = race();
    vec![
        &race.shared.car.tyre_change_state as &dyn Cell32,
        &race.shared.physics_tables.input_gear_down,
        &race.shared.physics_tables.input_gear_up,
        &race.shared.physics_tables.input_throttle,
        &race.shared.physics_tables.input_brake,
        &race.shared.physics_tables.input_handbrake,
        &race.shared.car.gear,
        &race.shared.car.gearbox_kind,
        &race.shared.physics_tables.input_gear_wanted,
        &race.shared.car.run_ticks,
        &race.shared.vars.run_kind,
        &race.shared.physics_tables.input_steer,
        &race.shared.car.go_stand_timer,
        &race.shared.car.started_stand_timer,
        &race.shared.car.finish_flag,
    ]
}

/// The Rust version.
pub fn gear_input_rules() {
    rules_at(&game());
}
