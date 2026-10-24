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

use crate::audio::store::Slot;
use crate::framework::rva;
use crate::sound::store::EffectsSlot;
use crate::store::{game_addr, game_read, game_write};

// ---- game data ----

const APPLIED_RVA: EffectsSlot = EffectsSlot::at(0x518080); // dword_518080
const CLEARED_RVA: Slot = Slot::at(0x506C84); // dword_506C84
const FIRST_RVA: Slot = Slot::at(0x4F82C4); // dword_4F82C4
const THIRD_RVA: usize = rva(0x511AE4); // unk_511AE4
const SECOND_RVA: usize = rva(0x755684); // dword_755684

// ---- Rust version ----

pub use rc2k_core::sound::set_up::Step;

/// A step run: each command sent with its eax, all the audio thread reads of it (the
/// original carries eax on to the steps after it, which read no more).
pub fn run_step(step: Step) -> Result<(), crate::errors::Refused> {
    match step {
        Step::StopMusic => crate::sound::stop_music::stop_music(),
        Step::Send { command, eax } => crate::sound::senders::send(command, [eax, 0, 0, 0]),
        Step::LoadRaceSounds => crate::sound::load_race_sounds::load_race_sounds_in_game()?,
        Step::Wait => crate::sound::wait_for_commands::wait_for_commands(
            &mut crate::sound::wait_for_commands::Game([0; 7]),
        ),
    }
    Ok(())
}

pub fn apply_settings() -> Result<(), crate::errors::Refused> {
    // Converted: `rc2k_core::sound::set_up::menus`.
    use rc2k_core::sound::set_up::{MENUS, menus};
    unsafe {
        game_write(APPLIED_RVA, MENUS);
        let steps = menus(
            game_read(FIRST_RVA),
            game_addr(THIRD_RVA) as u32,
            game_read(SECOND_RVA),
        );
        for s in steps {
            run_step(s)?;
        }
        game_write(CLEARED_RVA, 0u32);
    }
    Ok(())
}
