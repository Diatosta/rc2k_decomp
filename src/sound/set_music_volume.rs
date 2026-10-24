//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::sound::senders::*;
use crate::store::game_read;

/// Nothing if the volume is the one in use.
pub fn set_music_volume(volume: u32) {
    // Converted: `rc2k_core::sound::senders::new_music_volume`.
    if rc2k_core::sound::senders::new_music_volume(volume, unsafe {
        game_read::<u32>(crate::audio::stream::MUSIC_STREAM_RVA + 4usize)
    }) {
        send_and_wait(0xB, [0, 0, volume, 3]);
    }
}
