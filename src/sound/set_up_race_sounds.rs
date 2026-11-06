//! `sound::set_up_race_sounds` (0x40F02E): puts the audio thread in its mode for a race.
//!
//! In:  nothing (the other registers go into the commands as they are)
//! Out: every register preserved
//! `dword_518080` = 2; `sound::stop_music`; command 5 with the race's sound mode
//! `dword_4F82C8`; `sound::load_race_sounds`; `sound::wait_for_commands`; `dword_506C84`
//! = 0 (`sub_409B76`). The menus' counterpart is `sound::apply_settings`.
//! Nine callers (the events' starters, before a stage is loaded).

use crate::sound::apply_settings::run_step;

// ---- Rust version ----

pub fn set_up_race_sounds() -> Result<(), crate::errors::Refused> {
    // Converted: `rc2k_core::sound::set_up::race`.
    use rc2k_core::sound::set_up::{RACE, race};
    crate::sound::owner::sound()
        .menu
        .settings_applied
        .done
        .set(RACE);
    for s in race(crate::audio::owner::audio().state.race_sound_mode.get()) {
        run_step(s)?;
    }
    crate::audio::owner::audio().state.sounds_held_back.set(0);
    Ok(())
}
