//! `sound::apply_settings` (0x40EFF3): sends the kept sound settings to the audio thread.
//!
//! `dword_518080` = 1; `sound::stop_music`; command 5 with `dword_4F82C4`; command 3 with the
//! menus' list (`unk_511AE4`, by handle); command 8 with `dword_755684`; `sound::wait_for_commands`;
//! `dword_506C84` = 0 (`sound::let_sounds_play`).
//! What the two numbers are is a reading: volumes, the second the one the options screen
//! steps between 0 and 0x40 (0x467F00, 0x467F24).
//! Five callers; `system::main_loop` calls it once the window is there.

use crate::audio::owner::audio;
use rc2k_core::audio::commands::SoundList;

pub use rc2k_core::sound::set_up::Step;

/// A step run: each command sent with its eax, all the audio thread reads of it (the
/// original carries eax on to the steps after it, which read no more).
pub fn run_step(
    step: Step,
    sound: &crate::sound::owner::Sound,
) -> Result<(), crate::errors::Refused> {
    let audio = crate::audio::owner::audio();
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
    let sound = crate::sound::owner::sound();
    let audio = audio();
    // Converted: `rc2k_core::sound::set_up::menus`.
    use rc2k_core::sound::set_up::{MENUS, menus};

    sound.menu.settings_applied.done.set(MENUS);
    let steps = menus(
        audio.state.menu_sound_mode.get(),
        SoundList::Menu,
        crate::audio::owner::audio()
            .levels
            .master_volume_shown
            .get(),
    );
    for s in steps {
        run_step(s, sound)?;
    }
    audio.state.sounds_held_back.set(0);

    Ok(())
}
