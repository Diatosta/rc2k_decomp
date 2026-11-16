//! `sound::set_up_race_sounds`: puts the audio thread in its mode for a race.
//!
//! Marks the settings applied as 2, stops the music, sends the race's sound mode
//! (`AudioShared.race_sound_mode`), loads the race sounds (`sound::load_race_sounds`), waits for
//! the commands (`sound::wait_for_commands`) and lets sounds play (`sound::let_sounds_play`). The
//! menus' counterpart is `sound::apply_settings`. Nine callers (the events' starters, before a
//! stage is loaded).

use crate::owners::Owners;
use crate::sound::apply_settings::run_step;

pub fn set_up_race_sounds(o: Owners<'static>) -> Result<(), crate::errors::Refused> {
    // The logic is `rc2k_core::sound::set_up::race`.
    use rc2k_core::sound::set_up::{RACE, race};
    o.sound.menu.settings_applied.get().done.set(RACE);
    for s in race(o.audio.state.race_sound_mode.get()) {
        run_step(s, o)?;
    }
    o.audio.state.sounds_held_back.set(0);
    Ok(())
}
