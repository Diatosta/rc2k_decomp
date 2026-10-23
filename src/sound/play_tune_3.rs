//!
//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::framework::Regs;

pub fn play_tune_3() {
    crate::sound::senders::play_tune(3);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    play_tune_3();
    Ok(())
}
