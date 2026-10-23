//!
//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::framework::Regs;

pub fn stop_stream() {
    crate::sound::senders::send_and_wait(9, [0; 4]);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    stop_stream();
    Ok(())
}
