//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::Command`); the registers above are the original's,
//! for reference.

/// The master volume.
pub fn send_command_8(volume: u32) {
    crate::sound::senders::send(rc2k_core::audio::commands::Command::Volume(volume));
}
