//! `drive::race_cars_step` (0x4608BB): every car's physics step. `drive::choose_shown_cars`; for
//! each car n of `g_raceCars` (`g_currentCar` = n, from 0), the four `input_words` .. 0, its block
//! (`g_carBlocks[n & 7]`) into `work_block`, `drive::load_car`; with the car running
//! (`driver_kind` set) `drive::read_car_controls`, `physics::gear_input_rules` and
//! `physics::car_physics_step`; `drive::store_car`. Then `drive::animate_stage` and
//! `drive::race_mode_cars`.
//!
//! Caller: `drive::race_init` (its fifty settling steps). Refuses (before anything) a cars'
//! count of 0 or past 8 (the original's `jnz` loop runs on), and what its calls refuse; the
//! cars' blocks, the work car and the physics' variables are put back then. The pace notes'
//! sound cues (`physics::pace_notes_step` -> the sound objects' queues) are not put back
//! between the two runs: a mismatch there would show in those queues only.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::race_steps;

/// One car's step (0x4608CA ..).
fn car_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (scene, net) = (o.scene, o.net);
    for word in &race.shared.physics_tables.input_words {
        word.set(0);
    }
    let slot = race_steps::block_slot(race.shared.work.current_car.get());
    race.shared
        .work
        .work_block
        .set(race.shared.work.car_block(slot));
    crate::drive::load_car::load_car(&race.shared)?;
    if race.shared.car.driver_kind.get() != 0 {
        crate::drive::read_car_controls::read_car_controls(race, net, scene)?;
        crate::physics::gear_input_rules::gear_input_rules(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.shared.vars,
        );
        crate::physics::car_physics_step::car_physics_step(o, race)?;
    }
    crate::drive::store_car::store_car(&race.shared)?;
    Ok(())
}

// Converted: `rc2k_core::drive::race_steps::{check_cars, each_car, block_slot}`.
pub fn race_cars_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (scene, net) = (o.scene, o.net);
    let shared = &race.shared;
    race_steps::check_cars(shared.view.race_cars.get())?;
    crate::drive::choose_shown_cars::choose_shown_cars_in(race, net)?;
    race_steps::each_car(
        || shared.work.current_car.get(),
        || shared.view.race_cars.get(),
        |n| shared.work.current_car.set(n),
        || car_step(o, race),
    )?;
    crate::drive::animate_stage::animate_stage(race, scene)?;
    crate::drive::race_mode_cars::race_mode_cars(race, scene)?;
    Ok(())
}
