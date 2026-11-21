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

use crate::physics::car_store::Car;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::gearbox::{GearCar, GearInputs};

/// The rules on the car's inputs and gear.
// Core: `rc2k_core::physics::gearbox::GearInputs::apply_rules`.
pub fn gear_input_rules(car: &Car, inputs: &mut GearInputs, vars: &RaceVars) {
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
    let mut gear = car.gear.get();
    inputs.apply_rules(&mut gear, &state);
    if gear != car.gear.get() {
        car.gear.set(gear);
    }
}
