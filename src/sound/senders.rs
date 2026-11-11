use crate::sound::queue::send_command;
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use rc2k_core::audio::commands::Command;
use rc2k_core::sound::senders::Music;

/// `command` queued.
pub fn send(command: Command) {
    send_command(command);
}

/// `send`, then `sound::wait_for_commands`.
pub fn send_and_wait(command: Command) {
    send(command);
    wait_for_commands(&mut Game);
}

/// Command 0xB (converted: `rc2k_core::sound::senders::Music`), sent and waited for.
pub fn music(m: Music) {
    send_and_wait(Command::Music {
        tune: m.tune,
        next: m.then,
        volume: m.volume,
        action: m.action,
    });
}

/// `sound::play_tune_n` (converted: `rc2k_core::sound::senders::tune_command`).
pub fn play_tune(n: u32) {
    let options = &crate::sound::owner::sound().menu.options;
    if let Some(m) = rc2k_core::sound::senders::tune_command(
        n,
        options.music_volume.get(),
        options.race_music_volume.get(),
        options.race_music_on.get() == 1,
    ) {
        music(m);
    }
}
