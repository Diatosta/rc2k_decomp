//! `sound::apply_settings` (0x40EFF3): sends the kept sound settings to the audio thread.
//!
//! In:  nothing (the other registers go into the commands as they are)
//! Out: every register preserved
//! `dword_518080` = 1; `sound::stop_music`; command 5 with `dword_4F82C4`; command 3 with the
//! address of `unk_511AE4`; command 8 with `dword_755684`; `sound::wait_for_commands`;
//! `dword_506C84` = 0 (`sub_409B76`).
//! What the two numbers are is a reading: volumes, the second the one the options screen
//! steps between 0 and 0x40 (`sub_467F00`, `sub_467F24`).
//! Five callers; `system::main_loop` calls it once the window is there.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::sound::queue::{Queue, send_command, sent};
use crate::sound::stop_music::stop_music_in_game;
use crate::sound::wait_for_commands::{Game, wait_for_commands};

// ---- game data ----

const APPLIED_RVA: usize = rva(0x518080); // dword_518080
const CLEARED_RVA: usize = rva(0x506C84); // dword_506C84
const FIRST_RVA: usize = rva(0x4F82C4); // dword_4F82C4
const THIRD_RVA: usize = rva(0x511AE4); // unk_511AE4
const SECOND_RVA: usize = rva(0x755684); // dword_755684

// ---- Rust version ----

pub use rc2k_core::sound::set_up::Step;

/// The steps with the registers each gets: the caller's, eax set by each command for it
/// and those after it.
pub fn run_steps(entry: &Regs, steps: &[Step], step: &mut dyn FnMut(Step, &Regs)) {
    let mut r = *entry;
    for &s in steps {
        if let Step::Send { eax, .. } = s {
            r.eax = eax;
        }
        step(s, &r);
    }
}

pub unsafe fn apply_settings(entry: &Regs, step: &mut dyn FnMut(Step, &Regs)) {
    // Converted: `rc2k_core::sound::set_up::menus`.
    use rc2k_core::sound::set_up::{MENUS, menus};
    unsafe {
        game_write(APPLIED_RVA, MENUS);
        let steps = menus(
            game_read(FIRST_RVA),
            game_addr(THIRD_RVA) as u32,
            game_read(SECOND_RVA),
        );
        run_steps(entry, &steps, step);
        game_write(CLEARED_RVA, 0u32);
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe {
        apply_settings(r, &mut |step, regs| match step {
            Step::StopMusic => stop_music_in_game(regs),
            Step::Send { command, .. } => send_command(&Queue::game(), command, sent(regs)),
            Step::LoadRaceSounds => {}
            Step::Wait => wait_for_commands(&mut Game(sent(regs))),
        });
    }
    Ok(())
}
