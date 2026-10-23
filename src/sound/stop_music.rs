//!
//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::framework::Regs;

pub fn stop_music() {
    // Converted: `rc2k_core::sound::senders::Music::STOP`.
    crate::sound::senders::music(rc2k_core::sound::senders::Music::STOP);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    stop_music();
    Ok(())
}
