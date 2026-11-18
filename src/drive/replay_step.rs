//! The two-player / replay race's step (`drive::race_tick`'s): `drive::choose_shown_cars`, the
//! replay's frame kept and, playing, read; the skid marks begun again at a frame change; then
//! for each car its input words (recorded or played), `drive::load_car`, and, when running,
//! controls, gear rules, steering, pedals, the reset key, `physics::car_physics_step`, the
//! network's step, the clocks + 1 and `drive::store_car`; then the stage's animation, the
//! race-mode cars, the duel's sound levels, the count down's beeps, the two cars' order, the hold
//! count and the replay's writing and clock. Refuses (before anything) a cars' count of 0 or past
//! 8, and what its calls refuse (the cars' blocks, the work car and the physics' variables put
//! back).

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::race_steps;

/// One car's step.
fn car_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (scene, input, net) = (o.scene, o.input, o.net);
    crate::drive::car_input_words::car_input_words(race, input, net)?;
    crate::drive::record_inputs_if::record_inputs_if(race, net)?;
    crate::drive::play_inputs_if::play_inputs_if(race)?;
    let slot = race_steps::block_slot(race.shared.work.get().current_car.get());
    race.shared
        .work
        .get()
        .work_block
        .set(race.shared.work.get().car_block(slot as usize));
    crate::drive::load_car::load_car(&race.shared)?;
    if race.shared.car().driver_kind.get() != 0 {
        crate::drive::read_car_controls::read_car_controls(race, scene, net)?;
        crate::physics::gear_input_rules::gear_input_rules(
            race.shared.car(),
            race.shared.physics_tables.get(),
            race.shared.vars.get(),
        );
        crate::drive::steer_inputs::steer_inputs(race, scene);
        crate::drive::pedal_inputs::pedal_inputs(race);
        crate::drive::reset_request::reset_request(race, scene)?;
        crate::physics::car_physics_step::car_physics_step(o, race)?;
        crate::net::race_step::race_step(race, net)?;
    }
    race.shared
        .car()
        .step_clock
        .set(race.shared.car().step_clock.get().wrapping_add(1));
    race.shared
        .car()
        .run_ticks
        .set(race.shared.car().run_ticks.get().wrapping_add(1));
    crate::drive::store_car::store_car(&race.shared)?;
    crate::drive::replay_v2_step::after_step(race)?;
    Ok(())
}

pub fn replay_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (sound, scene) = (o.sound, o.scene);
    let shared = &race.shared;
    race_steps::check_cars(shared.view.get().race_cars.get())?;
    crate::drive::choose_shown_cars::choose_shown_cars_in(race, o.net)?;
    crate::drive::keep_replay_frame::keep_replay_frame(race)?;
    crate::drive::play_replay_frame::play_replay_frame(race, o.system)?;
    if race_steps::skids_restart(
        race.car_blocks().cars[0].step_clock.get(),
        race.car_blocks().cars[0].previous_step_clock.get(),
    ) {
        crate::race::stage_prefs::init_skidmarks(race)?;
    }
    race.car_blocks().cars[0]
        .previous_step_clock
        .set(race.car_blocks().cars[0].step_clock.get());
    race_steps::each_car(
        || shared.work.get().current_car.get(),
        || shared.view.get().race_cars.get(),
        |n| shared.work.get().current_car.set(n),
        || car_step(o, race),
    )?;
    crate::drive::replay_check::note(race)?;
    crate::drive::animate_stage::animate_stage(race, scene)?;
    crate::drive::race_mode_cars::race_mode_cars(race, scene)?;
    crate::drive::duel_sound_levels::duel_sound_levels(race, sound);
    crate::drive::countdown_beeps::countdown_beeps(race, o.audio)?;
    crate::drive::two_car_order::two_car_order(race)?;
    crate::drive::flush_replay_if::flush_replay_if(race, o.system, o.net)?;
    crate::drive::replay_clock_tick::replay_clock_tick(&race.replay);
    Ok(())
}
