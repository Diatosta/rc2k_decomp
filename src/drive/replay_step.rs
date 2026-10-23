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
//! Caller: `drive::race_step`'s `sub_4455A9`. It begins with a call: hooked by a private
//! copy of its 0xE1 bytes (its 25 calls relocated) and a hand-made patch.
//! Refuses (before anything) a cars' count of 0 or past 8, and what its calls refuse
//! (the cars' blocks, the work car and the physics' variables put back). Shadow mode runs
//! the original alone (as `car_physics_step`).
//! Self-test (in a race; skipped otherwise): the original and the Rust version from the
//! same state, logging off, no replay (`dword_5C5CE0` 0).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ESI, Regs};
use crate::physics::flow::{g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use rc2k_core::drive::race_steps;

const B: usize = 0x70F3EC; // g_carBlockStore
const CURRENT: usize = 0x71B3F8;
const CARS: usize = 0x63DB90;

fn car_step(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::drive::car_input_words::car_input_words(r)?.apply(r);
        crate::drive::record_inputs_if::record_inputs_if().apply(r);
        crate::drive::play_inputs_if::play_inputs_if(r).apply(r);
        r.eax = race_steps::block_slot(g(CURRENT)) as u32;
        r.esi = g(0x71B3B8 + 4 * r.eax as usize);
        p(0x71B404, r.esi);
        crate::drive::load_car::load_car()?.apply(r);
        if g(0x71A234) != 0 {
            let mut c = *r;
            crate::drive::read_car_controls::read_car_controls(&mut c)?;
            Left::nothing().with(EAX, c.eax).with(ESI, c.esi).apply(r);
            let mut c = *r;
            crate::physics::gear_input_rules::gear_input_rules(&mut c);
            r.eax = c.eax;
            crate::drive::steer_inputs::steer_inputs(r).apply(r);
            crate::drive::pedal_inputs::pedal_inputs().apply(r);
            *r = crate::drive::reset_request::reset_request(r)?;
            crate::physics::car_physics_step::car_physics_step(r)?.apply(r);
            crate::net::race_step::race_step(r, &mut crate::net::game_part)?;
        }
        p(0x719FE0, g(0x719FE0).wrapping_add(1));
        p(0x719FEC, g(0x719FEC).wrapping_add(1));
        crate::drive::store_car::store_car()?.apply(r);
    }
    Ok(())
}

/// The steps; `r` as the original leaves it.
// Converted: `rc2k_core::drive::race_steps::{check_cars, each_car, block_slot,
// skids_restart}`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::drive::choose_shown_cars::choose_shown_cars()?.apply(r);
        crate::drive::keep_replay_frame::keep_replay_frame().apply(r);
        crate::drive::play_replay_frame::play_replay_frame(r)?.apply(r);
    }
    r.eax = g(B + 8).wrapping_sub(g(B + 0xF0C));
    if race_steps::skids_restart(g(B + 8), g(B + 0xF0C)) {
        *r = unsafe { crate::race::stage_prefs::init_skidmarks(r) };
    }
    r.eax = g(B + 8);
    p(B + 0xF0C, r.eax);
    r.eax = race_steps::each_car(
        || g(CURRENT),
        || g(CARS),
        |n| p(CURRENT, n),
        || car_step(&mut r.clone()),
    )?;
    unsafe {
        crate::drive::animate_stage::animate_stage(r)?;
        *r = crate::drive::race_mode_cars::race_mode_cars(r, &mut |_, r| {
            *r = crate::drive::start_cars_behind::start_cars_behind(
                r,
                &mut crate::drive::start_cars_behind::game_part,
            )?;
            Ok(())
        })?;
        crate::drive::duel_sound_levels::duel_sound_levels(r).apply(r);
        crate::drive::countdown_beeps::countdown_beeps(r)?.apply(r);
        crate::drive::two_car_order::two_car_order(r).apply(r);
        crate::drive::hold_count_tick::hold_count_tick();
        crate::drive::flush_replay_if::flush_replay_if(r)?.apply(r);
        crate::drive::replay_clock_tick::replay_clock_tick();
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn replay_step(entry: &Regs) -> Result<Left, Refused> {
    race_steps::check_cars(g(CARS))?;
    let mut r = *entry;
    match steps(&mut r) {
        Ok(()) => Ok(left_of(&r)),
        Err(e) => Err(e),
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { replay_step(r) }?;
    left.apply(r);
    Ok(())
}
