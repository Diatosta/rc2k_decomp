//! `drive::race_cars_step` (0x4608BB): every car's physics step. `drive::choose_shown_cars`;
//! for each car n of `g_raceCars` (`g_currentCar` = n, from 0): in `pusha` / `popa`, the
//! four `dword_6138C4` .. 0, its block (`g_carBlocks[n & 7]`) into `dword_71B404`,
//! `drive::load_car`; with the car running (`unk_71A234` set) `drive::read_car_controls`,
//! `physics::gear_input_rules` and `physics::car_physics_step`; `drive::store_car`. Then
//! `drive::animate_stage` and `drive::race_mode_cars`.
//!
//! In:  nothing
//! Out: eax the cars' count, then as `animate_stage` and `race_mode_cars` leave the
//!      registers; no flags read
//!
//! Caller: `drive::race_init` (its fifty settling steps). Refuses (before anything) a cars'
//! count of 0 or past 8 (the original's `jnz` loop runs on), and what its calls refuse; the
//! cars' blocks, the work car and the physics' variables are put back then. The pace notes'
//! sound cues (`physics::pace_notes_step` -> the sound objects' queues) are not put back
//! between the two runs: a mismatch there would show in those queues only.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::race_steps;

/// One car's step (0x4608CA .., inside `pusha` / `popa`).
fn car_step() -> Result<(), Refused> {
    for word in &PHYSICS.input_words {
        word.set(0);
    }
    let slot = race_steps::block_slot(RACE_WORK.current_car.get());
    RACE_WORK.work_block.set(RACE_WORK.car_block(slot));
    crate::drive::load_car::load_car()?;
    if CAR.driver_kind.get() != 0 {
        crate::drive::read_car_controls::read_car_controls()?;
        crate::physics::gear_input_rules::gear_input_rules();
        crate::physics::car_physics_step::car_physics_step()?;
    }
    crate::drive::store_car::store_car()?;
    Ok(())
}

/// The Rust version. The registers it leaves are read by no caller (liveness: only DF
/// is live after `drive::race_init`'s call).
// Converted: `rc2k_core::drive::race_steps::{check_cars, each_car, block_slot}`.
pub unsafe fn race_cars_step() -> Result<(), Refused> {
    race_steps::check_cars(RACE_VIEW.race_cars.get())?;
    unsafe {
        crate::drive::choose_shown_cars::choose_shown_cars()?;
        race_steps::each_car(
            || RACE_WORK.current_car.get(),
            || RACE_VIEW.race_cars.get(),
            |n| RACE_WORK.current_car.set(n),
            car_step,
        )?;
        crate::drive::animate_stage::animate_stage()?;
        crate::drive::race_mode_cars::race_mode_cars()?;
    }
    Ok(())
}
