//! `drive::replay_step` (0x460B2F): the two-player / replay race's step (`sub_4455A9`'s).
//! `drive::choose_shown_cars`; the replay's frame kept (`drive::keep_replay_frame`) and,
//! playing, read (`drive::play_replay_frame`); the skid marks begun again
//! (`race::init_skidmarks`) unless the first block's clock (+8) is one past +0xF0C, which
//! takes it. For each car (`g_currentCar` from 0, in `pusha` / `popa`): its input words
//! (`drive::car_input_words`, recorded or played: `record_inputs_if`, `play_inputs_if`),
//! its block into `dword_71B404`, `drive::load_car`; running (`unk_71A234`): controls,
//! gear rules, steering, pedals, the reset key, `physics::car_physics_step` and the
//! network's step (`net::race_step`, its Rust version); the work car's clocks
//! (`unk_719FE0`, `unk_719FEC`) + 1; `drive::store_car`. Then the stage's animation, the
//! race-mode cars, the duel's sound levels, the count down's beeps, the two cars' order,
//! the hold count, the replay written at its last tenth, the replay's clock.
//!
//! In:  nothing
//! Out: the registers as the last steps leave them; no flags read
//!
//! Caller: `drive::race_step`'s `sub_4455A9`. Refuses (before anything) a cars' count of 0
//! or past 8, and what its calls refuse (the cars' blocks, the work car and the physics'
//! variables put back).

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::race_steps;

/// One car's step (inside `pusha` / `popa`: nothing it leaves in the registers goes out).
fn car_step() -> Result<(), Refused> {
    unsafe {
        crate::drive::car_input_words::car_input_words()?;
        crate::drive::record_inputs_if::record_inputs_if()?;
        crate::drive::play_inputs_if::play_inputs_if()?;
        let slot = race_steps::block_slot(RACE_WORK.current_car.get());
        RACE_WORK.work_block.set(RACE_WORK.car_block(slot as usize));
        crate::drive::load_car::load_car()?;
        if CAR.driver_kind.get() != 0 {
            crate::drive::read_car_controls::read_car_controls()?;
            crate::physics::gear_input_rules::gear_input_rules();
            crate::drive::steer_inputs::steer_inputs();
            crate::drive::pedal_inputs::pedal_inputs();
            crate::drive::reset_request::reset_request()?;
            crate::physics::car_physics_step::car_physics_step()?;
            // `net::race_step` reads no register at entry (liveness: its esi "read" at
            // 0x4644AF is `imul esi, ebp, ..`'s destination); its Rust version still takes
            // the registers: a local scratch.
            crate::net::race_step::race_step()?;
        }
        CAR.step_clock.set(
            crate::physics::car_store::CAR
                .step_clock
                .get()
                .wrapping_add(1),
        );
        CAR.run_ticks.set(
            crate::physics::car_store::CAR
                .run_ticks
                .get()
                .wrapping_add(1),
        );
        crate::drive::store_car::store_car()?;
    }
    Ok(())
}

/// The Rust version. The registers it leaves are read by no caller (liveness: ecx, edi
/// and esi after `drive::race_tick`'s call have no read point).
// Converted: `rc2k_core::drive::race_steps::{check_cars, each_car, block_slot,
// skids_restart}`.
pub unsafe fn replay_step() -> Result<(), Refused> {
    race_steps::check_cars(RACE_VIEW.race_cars.get())?;
    unsafe {
        crate::drive::choose_shown_cars::choose_shown_cars()?;
        crate::drive::keep_replay_frame::keep_replay_frame()?;
        crate::drive::play_replay_frame::play_replay_frame()?;
        if race_steps::skids_restart(
            CAR_BLOCKS.cars[0].step_clock.bits(),
            CAR_BLOCKS.cars[0].unknown_f0c.bits(),
        ) {
            // `race::init_skidmarks` reads no register at entry (liveness): a scratch.
            crate::race::stage_prefs::init_skidmarks()?;
        }
        CAR_BLOCKS.cars[0]
            .unknown_f0c
            .set_bits(CAR_BLOCKS.cars[0].step_clock.bits());
        race_steps::each_car(
            || RACE_WORK.current_car.get(),
            || RACE_VIEW.race_cars.get(),
            |n| RACE_WORK.current_car.set(n),
            car_step,
        )?;
        crate::drive::replay_check::note()?;
        crate::drive::animate_stage::animate_stage()?;
        crate::drive::race_mode_cars::race_mode_cars()?;
        crate::drive::duel_sound_levels::duel_sound_levels();
        crate::drive::countdown_beeps::countdown_beeps()?;
        crate::drive::two_car_order::two_car_order()?;
        crate::drive::hold_count_tick::hold_count_tick();
        crate::drive::flush_replay_if::flush_replay_if()?;
        crate::drive::replay_clock_tick::replay_clock_tick();
    }
    Ok(())
}
