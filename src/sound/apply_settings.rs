//! `sound::apply_settings` (0x40EFF3): sends the kept sound settings to the audio thread.
//!
//! `SettingsApplied.done` = 1; `sound::stop_music`; command 5 with `AudioState.menu_sound_mode`; command 3 with the
//! menus' list (`unk_511AE4`, by handle); command 8 with `dword_755684`; `sound::wait_for_commands`;
//! `AudioState.sounds_held_back` = 0 (`sound::let_sounds_play`).
//! What the two numbers are is a reading: volumes, the second the one the options screen
//! steps between 0 and 0x40 (0x467F00, 0x467F24).
//! Five callers; `system::main_loop` calls it once the window is there.

use crate::owners::Owners;
use rc2k_core::audio::commands::SoundList;

pub use rc2k_core::sound::set_up::Step;

/// One step of the settings run: a command sent, the music stopped, the race sounds
/// loaded, or the queue waited for.
pub fn run_step(step: Step, o: Owners<'static>) -> Result<(), crate::errors::Refused> {
    match step {
        Step::StopMusic => crate::sound::stop_music::stop_music(o.audio, o.system),
        Step::Send(command) => crate::sound::senders::send(o.audio, command),
        Step::LoadRaceSounds => {
            crate::sound::load_race_sounds::load_race_sounds_in_game(o.race, o.sound, o.audio)?;
        }
        Step::Wait => crate::sound::wait_for_commands::wait_in_game(o.audio, o.system),
    }
    Ok(())
}

pub fn apply_settings(o: Owners<'static>) -> Result<(), crate::errors::Refused> {
    // Converted: `rc2k_core::sound::set_up::menus`.
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
