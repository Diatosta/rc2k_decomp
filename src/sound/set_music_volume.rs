//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::sound::senders::*;

/// Nothing if the volume is the one in use.
pub fn set_music_volume(volume: u32) {
    // Converted: `rc2k_core::sound::senders::new_music_volume`.
    if rc2k_core::sound::senders::new_music_volume(
        volume,
        crate::audio::owner::audio().state.music_stream.get().volume,
    ) {
        send_and_wait(0xB, [0, 0, volume, 3]);
    }
}
