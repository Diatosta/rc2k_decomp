//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

pub fn stop_music() {
    // Converted: `rc2k_core::sound::senders::Music::STOP`.
    crate::sound::senders::music(rc2k_core::sound::senders::Music::STOP);
}
