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
//!
//! Callers: race init's loop (`drive::race_cars_step`), `race::step_replay_cars`,
//! `drive::replay_step` (each calls `physics::car_physics_step` or `drive::steer_inputs` next).

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::vars_store::RaceVars;
use crate::session::DwordLike;
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
        w90: tables.input_steer.get(),
        first: tables.input_throttle.get(),
        second: tables.input_brake.get(),
        hold: tables.input_handbrake.get(),
        wa0: tables.input_gear_up.get(),
        wa4: tables.input_gear_down.get(),
        joined: tables.input_gear_wanted.get(),
    };
    let gear_before = car.gear.get();
    let (mut inputs, mut gear) = (before, gear_before);
    inputs.apply_rules(&mut gear, &state);
    let changes: [(&dyn DwordLike, u32, u32); 7] = [
        (&tables.input_steer, inputs.w90, before.w90),
        (&tables.input_throttle, inputs.first, before.first),
        (&tables.input_brake, inputs.second, before.second),
        (&tables.input_handbrake, inputs.hold, before.hold),
        (&tables.input_gear_up, inputs.wa0, before.wa0),
        (&tables.input_gear_down, inputs.wa4, before.wa4),
        (&tables.input_gear_wanted, inputs.joined, before.joined),
    ];
    for (cell, now, was) in changes {
        crate::physics::cells::set_changed(cell, was, now);
    }
    if gear != gear_before {
        car.gear.set(gear);
    }
}
