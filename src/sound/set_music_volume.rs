//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::Command`); the registers above are the original's,
//! for reference.

use crate::sound::senders::*;
use rc2k_core::audio::commands::Command;

/// Nothing if the volume is the one in use.
pub fn set_music_volume(volume: u32, audio: &crate::audio::owner::Audio) {
    // Converted: `rc2k_core::sound::senders::new_music_volume`.
    if rc2k_core::sound::senders::new_music_volume(volume, audio.state.music_stream.get().volume) {
        send_and_wait(Command::Music {
            tune: 0,
            next: 0,
            volume,
            action: 3,
        });
    }
}
