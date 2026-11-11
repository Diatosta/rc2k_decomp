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
//! Returns: the joined input when they were joined
//!
//! Callers: race init's loop (`drive::race_cars_step`), `race::step_replay_cars`,
//! `drive::replay_step` (each calls
//! `physics::car_physics_step` or `drive::steer_inputs` next).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::vars_store::RaceVars;
use crate::session::Cell32;
use rc2k_core::physics::gearbox::{GearCar, GearInputs};

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// On the variables' cells `v`; what the rules change is written back.
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

fn game<'a>(car: &'a Car, tables: &'a PhysicsTables, vars: &'a RaceVars) -> Vec<&'a dyn Cell32> {
    vec![
        &car.tyre_change_state as &dyn Cell32,
        &tables.input_gear_down,
        &tables.input_gear_up,
        &tables.input_throttle,
        &tables.input_brake,
        &tables.input_handbrake,
        &car.gear,
        &car.gearbox_kind,
        &tables.input_gear_wanted,
        &car.run_ticks,
        &vars.run_kind,
        &tables.input_steer,
        &car.go_stand_timer,
        &car.started_stand_timer,
        &car.finish_flag,
    ]
}

pub fn gear_input_rules(car: &Car, tables: &PhysicsTables, vars: &RaceVars) {
    rules_at(&game(car, tables, vars));
}
