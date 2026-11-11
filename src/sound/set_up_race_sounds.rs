//! `sound::set_up_race_sounds` (0x40F02E): puts the audio thread in its mode for a race.
//!
//! `SettingsApplied.done` = 2; `sound::stop_music`; command 5 with the race's sound mode
//! `AudioState.race_sound_mode`; `sound::load_race_sounds`; `sound::wait_for_commands`; `AudioState.sounds_held_back`
//! = 0 (`sound::let_sounds_play`). The menus' counterpart is `sound::apply_settings`.
//! Nine callers (the events' starters, before a stage is loaded).

use crate::audio::owner::audio;
use crate::sound::apply_settings::run_step;

pub fn set_up_race_sounds(
    sound: &crate::sound::owner::Sound,
) -> Result<(), crate::errors::Refused> {
    let audio = audio();
    // Converted: `rc2k_core::sound::set_up::race`.
    use rc2k_core::sound::set_up::{RACE, race};
    sound.menu.settings_applied.done.set(RACE);
    for s in race(audio.state.race_sound_mode.get()) {
        run_step(s, sound)?;
    }
    audio.state.sounds_held_back.set(0);
    Ok(())
}
