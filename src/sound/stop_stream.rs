//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::Command`); the registers above are the original's,
//! for reference.

pub fn stop_stream() {
    crate::sound::senders::send_and_wait(rc2k_core::audio::commands::Command::Stream {
        path: 0,
        volume: 0,
        action: 0,
    });
}
