//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

/// The sample file `name` (its `audio::store::name_key`; the original sent the name's
/// address) played at `volume`.
pub fn start_stream(name: u32, volume: u32) {
    crate::sound::senders::send_and_wait(9, [name, 0, volume, 2]);
}
