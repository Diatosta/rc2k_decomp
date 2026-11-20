use crate::sound::queue::send_command_in;
use crate::sound::wait_for_commands::{Game, wait_for_commands};
use rc2k_core::audio::commands::Command;
use rc2k_core::sound::senders::Music;

/// `command` queued.
pub fn send(audio: &crate::audio::owner::Audio, command: Command) {
    send_command_in(audio, command);
}

/// `send`, then `sound::wait_for_commands`.
pub(super) fn send_and_wait(o: crate::owners::Owners<'static>, command: Command) {
    send(o.audio, command);
    wait_for_commands(&mut Game { o });
}

/// Command 0xB (converted: `rc2k_core::sound::senders::Music`), sent and waited for.
pub fn music(o: crate::owners::Owners<'static>, m: Music) {
    send_and_wait(
        o,
        Command::Music {
            tune: m.tune,
            next: m.then,
            volume: m.volume,
            action: m.action,
        },
    );
}

/// `sound::play_tune_n` (converted: `rc2k_core::sound::senders::tune_command`).
pub fn play_tune(o: crate::owners::Owners<'static>, n: u32) {
    let options = &o.sound.menu.options;
    if let Some(m) = rc2k_core::sound::senders::tune_command(
        n,
        options.music_volume.get(),
        options.race_music_volume.get(),
        options.race_music_on.get() == 1,
    ) {
        music(o, m);
    }
}
