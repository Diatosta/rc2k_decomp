use crate::sound::queue::send_command_in;
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use rc2k_core::audio::commands::Command;
use rc2k_core::sound::senders::Music;

/// `command` queued.
pub fn send(audio: &crate::audio::owner::Audio, command: Command) {
    send_command_in(audio, command);
}

/// `send`, then `sound::wait_for_commands`.
pub fn send_and_wait(
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
    command: Command,
) {
    send(audio, command);
    wait_for_commands(&mut Game { audio, system });
}

/// Command 0xB (converted: `rc2k_core::sound::senders::Music`), sent and waited for.
pub fn music(audio: &crate::audio::owner::Audio, system: &crate::system::owner::System, m: Music) {
    send_and_wait(
        audio,
        system,
        Command::Music {
            tune: m.tune,
            next: m.then,
            volume: m.volume,
            action: m.action,
        },
    );
}

/// `sound::play_tune_n` (converted: `rc2k_core::sound::senders::tune_command`).
pub fn play_tune(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
    n: u32,
) {
    let options = &sound.menu.options;
    if let Some(m) = rc2k_core::sound::senders::tune_command(
        n,
        options.music_volume.get(),
        options.race_music_volume.get(),
        options.race_music_on.get() == 1,
    ) {
        music(audio, system, m);
    }
}
