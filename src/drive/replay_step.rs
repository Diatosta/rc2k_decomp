//! `drive::replay_step` (0x460B2F): the two-player / replay race's step (`drive::race_tick`'s).
//! `drive::choose_shown_cars`; the replay's frame kept (`drive::keep_replay_frame`) and, playing,
//! read (`drive::play_replay_frame`); the skid marks begun again (`race::init_skidmarks`) unless
//! the first block's clock (+8) is one past +0xF0C, which takes it. For each car (`g_currentCar`
//! from 0): its input words (`drive::car_input_words`, recorded or played: `record_inputs_if`,
//! `play_inputs_if`), its block into `work_block`, `drive::load_car`; running (`driver_kind`):
//! controls, gear rules, steering, pedals, the reset key, `physics::car_physics_step` and the
//! network's step (`net::race_step`, its Rust version); the work car's clocks (`step_clock`,
//! `run_ticks`) + 1; `drive::store_car`. Then the stage's animation, the race-mode cars, the
//! duel's sound levels, the count down's beeps, the two cars' order, the hold count, the replay
//! written at its last tenth, the replay's clock.
//!
//! Caller: `drive::race_step`'s `drive::race_tick`. Refuses (before anything) a cars' count of 0
//! or past 8, and what its calls refuse (the cars' blocks, the work car and the physics'
//! variables put back).

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::race_steps;

/// One car's step (inside `pusha` / `popa`: nothing it leaves in the registers goes out).
fn car_step(
    race: &Race,
    system: &crate::system::owner::System,
    sound: &crate::sound::owner::Sound,
    scene: &crate::scene::owner::Scene,
    input: &crate::input::owner::Input,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    crate::drive::car_input_words::car_input_words(race, input, net)?;
    crate::drive::record_inputs_if::record_inputs_if(race)?;
    crate::drive::play_inputs_if::play_inputs_if(race)?;
    let slot = race_steps::block_slot(race.shared.work.current_car.get());
    race.shared
        .work
        .work_block
        .set(race.shared.work.car_block(slot as usize));
    crate::drive::load_car::load_car(&race.shared)?;
    if race.shared.car.driver_kind.get() != 0 {
        crate::drive::read_car_controls::read_car_controls(race, net, scene)?;
        crate::physics::gear_input_rules::gear_input_rules(
            &race.shared.car,
            &race.shared.physics_tables,
            &race.shared.vars,
        );
        crate::drive::steer_inputs::steer_inputs(race, scene);
        crate::drive::pedal_inputs::pedal_inputs(race);
        crate::drive::reset_request::reset_request(race, scene)?;
        crate::physics::car_physics_step::car_physics_step(race, system, sound, scene, net)?;
        crate::net::race_step::race_step(race, net)?;
    }
    race.shared
        .car
        .step_clock
        .set(race.shared.car.step_clock.get().wrapping_add(1));
    race.shared
        .car
        .run_ticks
        .set(race.shared.car.run_ticks.get().wrapping_add(1));
    crate::drive::store_car::store_car(&race.shared)?;
    Ok(())
}

// Converted: `rc2k_core::drive::race_steps::{check_cars, each_car, block_slot,
// skids_restart}`.
pub fn replay_step(
    race: &Race,
    system: &crate::system::owner::System,
    sound: &crate::sound::owner::Sound,
    scene: &crate::scene::owner::Scene,
    input: &crate::input::owner::Input,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    let shared = &race.shared;
    race_steps::check_cars(shared.view.race_cars.get())?;
    crate::drive::choose_shown_cars::choose_shown_cars_in(race, net)?;
    crate::drive::keep_replay_frame::keep_replay_frame(race)?;
    crate::drive::play_replay_frame::play_replay_frame(race)?;
    if race_steps::skids_restart(
        race.car_blocks().cars[0].step_clock.bits(),
        race.car_blocks().cars[0].previous_step_clock.bits(),
    ) {
        crate::race::stage_prefs::init_skidmarks(race)?;
    }
    race.car_blocks().cars[0]
        .previous_step_clock
        .set(race.car_blocks().cars[0].step_clock.get());
    race_steps::each_car(
        || shared.work.current_car.get(),
        || shared.view.race_cars.get(),
        |n| shared.work.current_car.set(n),
        || car_step(race, system, sound, scene, input, net),
    )?;
    crate::drive::replay_check::note(race)?;
    crate::drive::animate_stage::animate_stage(race, scene)?;
    crate::drive::race_mode_cars::race_mode_cars(race, scene)?;
    crate::drive::duel_sound_levels::duel_sound_levels(race, sound);
    crate::drive::countdown_beeps::countdown_beeps(race)?;
    crate::drive::two_car_order::two_car_order(race)?;
    crate::drive::hold_count_tick::hold_count_tick(&race.shared);
    crate::drive::flush_replay_if::flush_replay_if(race)?;
    crate::drive::replay_clock_tick::replay_clock_tick(&race.replay);
    Ok(())
}
