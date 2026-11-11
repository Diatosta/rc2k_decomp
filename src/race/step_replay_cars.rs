//! `race::step_replay_cars` (0x460835): one step of the replay's cars, for the viewer's still
//! frame: the cars shown (`drive::choose_shown_cars`); for each car (`g_currentCar` from 0) its
//! input words (`drive::car_input_words`, `drive::play_inputs`), its block into `work_block`, the
//! work car loaded (`drive::load_car`); running (`driver_kind`): controls, gear rules, steering,
//! pedals, the reset key, `physics::car_physics_step` and the work car's clocks (`step_clock`,
//! `run_ticks`) one more; `drive::store_car`. Then `drive::race_mode_cars`.
//!
//! Caller: `race::show_replay_still` (two). Unlike `drive::replay_step` the clocks count only
//! for a running car, and nothing is recorded or sent. Refuses (before anything) a cars' count
//! of 0 or past 8 (0 goes round 2^32 times).

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn step_replay_cars(
    race: &Race,
    input: &crate::input::owner::Input,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = &race.shared;
    use crate::drive::*;
    // Converted: `rc2k_core::race::viewer_flow::{check_replay_cars, car_block, car_runs,
    // RUN_STEPS, next_car}`.
    use rc2k_core::race::viewer_flow::{
        RUN_STEPS, RunStep, car_block, car_runs, check_replay_cars, next_car,
    };
    check_replay_cars(shared.view.race_cars.get())?;
    choose_shown_cars::choose_shown_cars(race)?;
    shared.work.current_car.set(0);
    loop {
        car_input_words::car_input_words(race, input)?;
        play_inputs::play_inputs(race)?;
        shared.work.work_block.set(
            shared
                .work
                .car_block(car_block(shared.work.current_car.get()) as usize),
        );
        load_car::load_car(shared)?;
        if car_runs(shared.car.driver_kind.get()) {
            for step in RUN_STEPS {
                match step {
                    RunStep::Controls => read_car_controls::read_car_controls(race)?,
                    RunStep::GearRules => crate::physics::gear_input_rules::gear_input_rules(
                        &race.shared.car,
                        &race.shared.physics_tables,
                        &race.shared.vars,
                    ),
                    RunStep::Steering => steer_inputs::steer_inputs(race),
                    RunStep::Pedals => pedal_inputs::pedal_inputs(race),
                    RunStep::ResetKey => {
                        reset_request::reset_request(race, scene)?;
                    }
                    RunStep::Physics => crate::physics::car_physics_step::car_physics_step(race)?,
                }
            }
            for clock in [&shared.car.step_clock, &shared.car.run_ticks] {
                clock.set(clock.get().wrapping_add(1));
            }
        }
        store_car::store_car(shared)?;
        let (next, done) = next_car(shared.work.current_car.get(), shared.view.race_cars.get());
        shared.work.current_car.set(next);
        if done {
            break;
        }
    }
    race_mode_cars::race_mode_cars(race)
}
