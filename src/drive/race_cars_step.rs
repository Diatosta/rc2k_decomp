//! Every car's physics step. `drive::choose_shown_cars`; for each car n (the current car = n,
//! from 0), the four `input_words` 0, its block into `work_block`, `drive::load_car`; with the
//! car running `drive::read_car_controls`, `physics::gear_input_rules` and
//! `physics::car_physics_step`; `drive::store_car`. Then `drive::animate_stage` and
//! `drive::race_mode_cars`.
//!
//! Refuses (before anything) a cars' count of 0 or past 8, and what its calls refuse; the cars'
//! blocks, the work car and the physics' variables are put back then.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::race_steps;

fn car_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let scene = o.scene;
    race.shared.run_mut().input_words = [0; 4];
    let slot = race_steps::block_slot(race.shared.work.get().current_car.get());
    race.shared
        .work
        .get()
        .work_block
        .set(race.shared.work.get().car_block(slot));
    crate::drive::load_car::load_car(&race.shared)?;
    if race.shared.car().driver_kind.get() != 0 {
        crate::drive::read_car_controls::read_car_controls(race, scene, o.net)?;
        crate::physics::gear_input_rules::gear_input_rules(
            race.shared.car(),
            &mut race.shared.run_mut().gear_inputs,
            race.shared.vars.get(),
        );
        crate::physics::car_physics_step::car_physics_step(o, race)?;
    }
    crate::drive::store_car::store_car(&race.shared)?;
    Ok(())
}

pub fn race_cars_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let scene = o.scene;
    let shared = &race.shared;
    race_steps::check_cars(shared.view.get().race_cars.get())?;
    crate::drive::choose_shown_cars::choose_shown_cars_in(race, o.net)?;
    race_steps::each_car(
        || shared.work.get().current_car.get(),
        || shared.view.get().race_cars.get(),
        |n| shared.work.get().current_car.set(n),
        || car_step(o, race),
    )?;
    crate::drive::animate_stage::animate_stage(race, scene)?;
    crate::drive::race_mode_cars::race_mode_cars(race, scene)?;
    Ok(())
}
