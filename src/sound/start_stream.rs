//! `sound::start_stream` (0x40F743): command 9 with edx = 2, and the wait for it.
//!
//! In:  eax, ecx (passed on in the command)
//! Out: every register preserved
//! Command 9 with eax and ecx as given, ebx = 0 and edx = 2, then
//! `sound::wait_for_commands`. The name is a guess, as `sound::stop_stream`'s. Seven
//! callers.
//! Shadow mode runs the original alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::senders::*;

// ---- Rust version ----

/// The registers the command is sent with.
pub fn start_stream(entry: &Regs) -> Option<Regs> {
    Some(Regs {
        ebx: 0,
        edx: 2,
        ..*entry
    })
}

pub const SENDER: Sender = Sender {
    command: 0x9,
    sends: start_stream,
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    send_entry(r, &SENDER)
}
