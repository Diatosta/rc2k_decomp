//! The gearbox's input state by the car's state (the work copy's `tyre_change_state`,
//! `gearbox_kind`, `run_ticks`, `go_stand_timer`, `started_stand_timer`, `finish_flag`): with
//! `tyre_change_state` neither 0 nor -1 all are cleared, the hold word = 500 and the gear is 0
//! (nothing else); with `gearbox_kind` 2 the two inputs are joined into the first (in neutral,
//! `input_gear_wanted` = 1, or -1 when the second is the larger) or swapped (in reverse); with
//! `run_ticks` below 800 the hold word = 500 and, unless `run_kind` is 1, four are cleared;
//! each of three switches clears them and the gear. What they are is a reading at best (an
//! automatic box's held inputs?).
//!
//! Returns the joined input when they were joined.

use crate::memory::session::DwordLike;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::gearbox::{GearCar, GearInputs};

/// The rules change what they change; only the words that differ are written back.
// Core: `rc2k_core::physics::gearbox::GearInputs::apply_rules`.
pub fn gear_input_rules(car: &Car, tables: &PhysicsTables, vars: &RaceVars) {
    let state = GearCar {
        state: car.tyre_change_state.get(),
        box_mode: car.gearbox_kind.get(),
        timer: car.run_ticks.get(),
        race_mode: vars.run_kind.get(),
        switches: [
            car.go_stand_timer.get(),
            car.started_stand_timer.get(),
            car.finish_flag.get(),
        ],
    };
    let before = GearInputs {
        steer: tables.input_steer.get(),
        first: tables.input_throttle.get(),
        second: tables.input_brake.get(),
        hold: tables.input_handbrake.get(),
        gear_up: tables.input_gear_up.get(),
        gear_down: tables.input_gear_down.get(),
        joined: tables.input_gear_wanted.get(),
    };
    let gear_before = car.gear.get();
    let (mut inputs, mut gear) = (before, gear_before);
    inputs.apply_rules(&mut gear, &state);
    let changes: [(&dyn DwordLike, u32, u32); 7] = [
        (&tables.input_steer, inputs.steer, before.steer),
        (&tables.input_throttle, inputs.first, before.first),
        (&tables.input_brake, inputs.second, before.second),
        (&tables.input_handbrake, inputs.hold, before.hold),
        (&tables.input_gear_up, inputs.gear_up, before.gear_up),
        (&tables.input_gear_down, inputs.gear_down, before.gear_down),
        (&tables.input_gear_wanted, inputs.joined, before.joined),
    ];
    for (cell, now, was) in changes {
        crate::physics::cells::set_changed(cell, was, now);
    }
    if gear != gear_before {
        car.gear.set(gear);
    }
}
