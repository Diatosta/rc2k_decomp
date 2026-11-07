//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::sound::senders::*;

/// Nothing if the volume is the one in use.
pub fn set_music_volume(volume: u32, audio: &crate::audio::owner::Audio) {
    // Converted: `rc2k_core::sound::senders::new_music_volume`.
    if rc2k_core::sound::senders::new_music_volume(volume, audio.state.music_stream.get().volume) {
        send_and_wait(0xB, [0, 0, volume, 3]);
    }
}
