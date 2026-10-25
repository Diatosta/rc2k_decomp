//! `sound::set_up_race_sounds` (0x40F02E): puts the audio thread in its mode for a race.
//!
//! In:  nothing (the other registers go into the commands as they are)
//! Out: every register preserved
//! `dword_518080` = 2; `sound::stop_music`; command 5 with the race's sound mode
//! `dword_4F82C8`; `sound::load_race_sounds`; `sound::wait_for_commands`; `dword_506C84`
//! = 0 (`sub_409B76`). The menus' counterpart is `sound::apply_settings`.
//! Nine callers (the events' starters, before a stage is loaded).

use crate::audio::store::Slot;
use crate::sound::apply_settings::run_step;
use crate::store::{game_read, game_write};

// ---- game data ----

const HELD_BACK_RVA: Slot = Slot::at(0x506C84); // dword_506C84
const RACE_MODE_RVA: Slot = Slot::at(0x4F82C8); // dword_4F82C8

// ---- Rust version ----

pub fn set_up_race_sounds() -> Result<(), crate::errors::Refused> {
    // Converted: `rc2k_core::sound::set_up::race`.
    use rc2k_core::sound::set_up::{RACE, race};
    unsafe {
        crate::sound::store::SOUND_EFFECTS
            .settings_applied
            .set(RACE);
        for s in race(game_read(RACE_MODE_RVA)) {
            run_step(s)?;
        }
        game_write(HELD_BACK_RVA, 0u32);
    }
    Ok(())
}
