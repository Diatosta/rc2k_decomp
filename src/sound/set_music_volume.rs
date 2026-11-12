/// Sets the music volume.
use crate::sound::senders::*;
use rc2k_core::audio::commands::Command;

/// Nothing if the volume is the one in use.
pub fn set_music_volume(
    volume: u32,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
) {
    // Converted: `rc2k_core::sound::senders::new_music_volume`.
    if rc2k_core::sound::senders::new_music_volume(volume, audio.state.music_stream.get().volume) {
        send_and_wait(
            audio,
            system,
            Command::Music {
                tune: 0,
                next: 0,
                volume,
                action: 3,
            },
        );
    }
}
