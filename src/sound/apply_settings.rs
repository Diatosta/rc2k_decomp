//! `sound::apply_settings` (0x40EFF3): sends the kept sound settings to the audio thread.
//!
//! `SettingsApplied.done` = 1; `sound::stop_music`; command 5 with `AudioState.menu_sound_mode`; command 3 with the
//! menus' list (`unk_511AE4`, by handle); command 8 with `dword_755684`; `sound::wait_for_commands`;
//! `AudioState.sounds_held_back` = 0 (`sound::let_sounds_play`).
//! What the two numbers are is a reading: volumes, the second the one the options screen
//! steps between 0 and 0x40 (0x467F00, 0x467F24).
//! Five callers; `system::main_loop` calls it once the window is there.

use crate::audio::owner::audio;
use rc2k_core::audio::commands::SoundList;

pub use rc2k_core::sound::set_up::Step;

/// One step of the settings run: a command sent, the music stopped, the race sounds
/// loaded, or the queue waited for.
pub fn run_step(
    step: Step,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), crate::errors::Refused> {
    match step {
        Step::StopMusic => crate::sound::stop_music::stop_music(),
        Step::Send(command) => crate::sound::senders::send(command),
        Step::LoadRaceSounds => {
            crate::sound::load_race_sounds::load_race_sounds_in_game(sound, audio)?;
        }
        Step::Wait => crate::sound::wait_for_commands::wait_for_commands(
            &mut crate::sound::wait_for_commands::Game,
        ),
    }
    Ok(())
}

pub fn apply_settings() -> Result<(), crate::errors::Refused> {
    apply_settings_in(crate::sound::owner::sound(), audio())
}

/// `apply_settings` for a caller that holds the owners.
pub fn apply_settings_in(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), crate::errors::Refused> {
    // Converted: `rc2k_core::sound::set_up::menus`.
    use rc2k_core::sound::set_up::{MENUS, menus};

    sound.menu.settings_applied.get().done.set(MENUS);
    let steps = menus(
        audio.state.menu_sound_mode.get(),
        SoundList::Menu,
        audio.levels.master_volume_shown.get(),
    );
    for s in steps {
        run_step(s, sound, audio)?;
    }
    audio.state.sounds_held_back.set(0);

    Ok(())
}
