//!
//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::framework::Regs;

/// The master volume.
pub fn send_command_8(volume: u32) {
    crate::sound::senders::send(8, [volume, 0, 0, 0]);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_command_8(r.eax);
    Ok(())
}
