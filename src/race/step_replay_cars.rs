//! `race::step_replay_cars`: one step of the replay's cars, for the viewer's still frame. The
//! cars shown are chosen (`drive::choose_shown_cars`). For each car (`current_car` from 0) its
//! input words are made (`drive::car_input_words`, `drive::play_inputs`), its block goes into
//! `work_block` and the work car is loaded (`drive::load_car`). A running car (`driver_kind`) then
//! gets controls, gear rules, steering, pedals, the reset key, `physics::car_physics_step` and
//! its clocks (`step_clock`, `run_ticks`) one more; `drive::store_car` follows. Last comes
//! `drive::race_mode_cars`.
//!
//! Unlike `drive::replay_step` the clocks count only for a running car, and nothing is recorded or
//! sent. Refuses (before anything) a cars' count of 0 or past 8 (a count of 0 would loop 2^32
//! times).

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn step_replay_cars(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (input, scene) = (o.input, o.scene);
    let shared = &race.shared;
    use crate::drive::*;
    use rc2k_core::race::viewer_flow::{
        RUN_STEPS, RunStep, car_block, car_runs, check_replay_cars, next_car,
    };
    check_replay_cars(shared.view.get().race_cars.get())?;
    choose_shown_cars::choose_shown_cars_in(race, o.net)?;
    shared.work.get().current_car.set(0);
    loop {
        car_input_words::car_input_words(race, input, o.net)?;
        play_inputs::play_inputs(race)?;
        shared.work.get().work_block.set(
            shared
                .work
                .get()
                .car_block(car_block(shared.work.get().current_car.get()) as usize),
        );
        load_car::load_car(shared)?;
        if car_runs(shared.car().driver_kind.get()) {
            for step in RUN_STEPS {
                match step {
                    RunStep::Controls => read_car_controls::read_car_controls(race, scene, o.net)?,
                    RunStep::GearRules => crate::physics::gear_input_rules::gear_input_rules(
                        race.shared.car(),
                        race.shared.physics_tables.get(),
                        race.shared.vars.get(),
                    ),
                    RunStep::Steering => steer_inputs::steer_inputs(race, scene),
                    RunStep::Pedals => pedal_inputs::pedal_inputs(race),
                    RunStep::ResetKey => {
                        reset_request::reset_request(race, scene)?;
                    }
                    RunStep::Physics => {
                        crate::physics::car_physics_step::car_physics_step(o, race)?;
                    }
                }
            }
            for clock in [&shared.car().step_clock, &shared.car().run_ticks] {
                clock.set(clock.get().wrapping_add(1));
            }
        }
        store_car::store_car(shared)?;
        replay_v2_step::after_step(race)?;
        let (next, done) = next_car(
            shared.work.get().current_car.get(),
            shared.view.get().race_cars.get(),
        );
        shared.work.get().current_car.set(next);
        if done {
            break;
        }
    }
    race_mode_cars::race_mode_cars(race, scene)
}
