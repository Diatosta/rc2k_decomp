//!
//! Plain since 2026-10-23 (phase 3): its arguments are what the audio thread reads of the
//! command (`rc2k_core::audio::commands::decode`); the registers above are the original's,
//! for reference.

use crate::framework::Regs;

/// The file at `path` played at `volume`.
pub fn start_stream(path: u32, volume: u32) {
    crate::sound::senders::send_and_wait(9, [path, 0, volume, 2]);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    start_stream(r.eax, r.ecx);
    Ok(())
}
