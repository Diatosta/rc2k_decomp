//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

/// The file at `path` played at `volume`.
pub fn start_stream(path: u32, volume: u32) {
    crate::sound::senders::send_and_wait(9, [path, 0, volume, 2]);
}
