//! `race::step_replay_cars` (0x460835): one step of the replay's cars, for the viewer's
//! still frame: the cars shown (`drive::choose_shown_cars`); for each car
//! (`g_currentCar` from 0, in `pusha` / `popa`) its input words (`drive::car_input_words`,
//! `drive::play_inputs`), its block into `dword_71B404`, the work car loaded
//! (`drive::load_car`); running (`unk_71A234`): controls, gear rules, steering, pedals,
//! the reset key, `physics::car_physics_step` and the work car's clocks (`unk_719FE0`,
//! `unk_719FEC`) one more; `drive::store_car`. Then `drive::race_mode_cars`.
//!
//! In:  nothing
//! Out: the registers as `race_mode_cars` leaves them; no flags read
//!
//! Caller: `race::show_replay_still` (two). Unlike `drive::replay_step` the clocks count only
//! for a running car, and nothing is recorded or sent. Refuses (before anything) a cars' count
//! of 0 or past 8 (0 goes round 2^32 times).

use crate::errors::Refused;
use crate::race::owner_shared::Shared;

/// The Rust version (the original leaves registers nothing reads).
pub fn step_replay_cars(shared: &Shared) -> Result<(), Refused> {
    use crate::drive::*;
    // Converted: `rc2k_core::race::viewer_flow::{check_replay_cars, car_block, car_runs,
    // RUN_STEPS, next_car}`.
    use rc2k_core::race::viewer_flow::{
        RUN_STEPS, RunStep, car_block, car_runs, check_replay_cars, next_car,
    };
    check_replay_cars(shared.view.race_cars.get())?;
    choose_shown_cars::choose_shown_cars()?;
    shared.work.current_car.set(0);
    loop {
        car_input_words::car_input_words()?;
        play_inputs::play_inputs()?;
        shared.work.work_block.set(
            shared
                .work
                .car_block(car_block(shared.work.current_car.get()) as usize),
        );
        load_car::load_car()?;
        if car_runs(shared.car.driver_kind.get()) {
            for step in RUN_STEPS {
                match step {
                    RunStep::Controls => read_car_controls::read_car_controls()?,
                    RunStep::GearRules => crate::physics::gear_input_rules::gear_input_rules(),
                    RunStep::Steering => steer_inputs::steer_inputs(),
                    RunStep::Pedals => pedal_inputs::pedal_inputs(),
                    RunStep::ResetKey => reset_request::reset_request()?,
                    RunStep::Physics => crate::physics::car_physics_step::car_physics_step()?,
                }
            }
            for clock in [&shared.car.step_clock, &shared.car.run_ticks] {
                clock.set(clock.get().wrapping_add(1));
            }
        }
        store_car::store_car()?;
        let (next, done) = next_car(shared.work.current_car.get(), shared.view.race_cars.get());
        shared.work.current_car.set(next);
        if done {
            break;
        }
    }
    race_mode_cars::race_mode_cars()
}
