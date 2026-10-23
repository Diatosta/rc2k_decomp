//! `sound::stop_stream` (0x40F757): command 9 with four zeros, and the wait for it.
//!
//! In:  nothing
//! Out: every register preserved
//! Command 9 (the audio thread's `sub_40C194` with eax, ecx, edx) with eax, ebx, ecx and
//! edx = 0, then `sound::wait_for_commands`. What command 9 plays is not read; "stream"
//! and "stop" are guesses from command 0xB's shape. Nine callers (the event loops, four
//! screen loops).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the command is sent with.
pub fn stop_stream(entry: &Regs) -> Option<Regs> {
    Some(Regs {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
        ..*entry
    })
}

const SENDER: Sender = Sender {
    command: 0x9,
    sends: stop_stream,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
