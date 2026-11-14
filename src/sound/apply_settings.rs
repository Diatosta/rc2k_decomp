//! `sound::apply_settings`: sends the kept sound settings to the audio thread.
//!
//! Marks the settings applied, stops the music, sends the menu sound mode, the menus' sound list
//! and the options screen's volume, waits for the commands, then lets sounds play. What the two
//! numbers are is a reading: volumes, the second the one the options screen steps between 0 and
//! 0x40. Five callers; `system::main_loop` calls it once the window is there.

use crate::owners::Owners;
use rc2k_core::audio::commands::SoundList;

pub use rc2k_core::sound::set_up::Step;

/// One step of the settings run: a command sent, the music stopped, the race sounds
/// loaded, or the queue waited for.
pub fn run_step(step: Step, o: Owners<'static>) -> Result<(), crate::errors::Refused> {
    match step {
        Step::StopMusic => crate::sound::stop_music::stop_music(o),
        Step::Send(command) => crate::sound::senders::send(o.audio, command),
        Step::LoadRaceSounds => {
            crate::sound::load_race_sounds::load_race_sounds_in_game(o.race, o.sound, o.audio)?;
        }
        Step::Wait => crate::sound::wait_for_commands::wait_in_game(o),
    }
    Ok(())
}

pub fn apply_settings(o: Owners<'static>) -> Result<(), crate::errors::Refused> {
    use rc2k_core::sound::set_up::{MENUS, menus};

    o.sound.menu.settings_applied.get().done.set(MENUS);
    let steps = menus(
        o.audio.state.menu_sound_mode.get(),
        SoundList::Menu,
        o.audio.levels.master_volume_shown.get(),
    );
    for s in steps {
        run_step(s, o)?;
    }
    o.audio.state.sounds_held_back.set(0);

    Ok(())
}
