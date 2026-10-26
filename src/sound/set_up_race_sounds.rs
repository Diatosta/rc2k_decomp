//! `sound::set_up_race_sounds` (0x40F02E): puts the audio thread in its mode for a race.
//!
//! In:  nothing (the other registers go into the commands as they are)
//! Out: every register preserved
//! `dword_518080` = 2; `sound::stop_music`; command 5 with the race's sound mode
//! `dword_4F82C8`; `sound::load_race_sounds`; `sound::wait_for_commands`; `dword_506C84`
//! = 0 (`sub_409B76`). The menus' counterpart is `sound::apply_settings`.
//! Nine callers (the events' starters, before a stage is loaded).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose four calls note the registers they get.

use crate::framework::{Regs, game_read, game_write, rva};
use crate::sound::apply_settings::{Step, run_steps};
use crate::sound::queue::{Queue, send_command, sent};
use crate::sound::stop_music::stop_music_in_game;
use crate::sound::wait_for_commands::{Game, wait_for_commands};

// ---- game data ----

const APPLIED_RVA: usize = rva(0x518080); // dword_518080
const HELD_BACK_RVA: usize = rva(0x506C84); // dword_506C84
const RACE_MODE_RVA: usize = rva(0x4F82C8); // dword_4F82C8
const LOAD_RVA: usize = rva(0x40EED8); // sound::load_race_sounds

// ---- Rust version ----

pub unsafe fn set_up_race_sounds(entry: &Regs, step: &mut dyn FnMut(Step, &Regs)) {
    // Converted: `rc2k_core::sound::set_up::race`.
    use rc2k_core::sound::set_up::{RACE, race};
    unsafe {
        game_write(APPLIED_RVA, RACE);
        run_steps(entry, &race(game_read(RACE_MODE_RVA)), step);
        game_write(HELD_BACK_RVA, 0u32);
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe {
        set_up_race_sounds(r, &mut |step, regs| match step {
            Step::StopMusic => stop_music_in_game(regs),
            Step::Send { command, .. } => send_command(&Queue::game(), command, sent(regs)),
            // Through its hook, which has the way out for what the Rust version refuses.
            Step::LoadRaceSounds => crate::calls::call(LOAD_RVA, &mut { *regs }),
            Step::Wait => wait_for_commands(&mut Game(sent(regs))),
        });
    }
    Ok(())
}
