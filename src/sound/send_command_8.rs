//! `sound::send_command_8` (0x409BB4): sends the audio thread command 0x8 with the
//! registers as they came, and does not wait.
//!
//! In:  the registers go into the command
//! Out: every register preserved
//! `pusha; push 8; call; pop eax; popa`. Callers: five (`sound::apply_settings`, 0x43A6F4, 0x43A735, 0x467F1E, 0x467F42).
//! A call in its first five bytes: private copy (0xB bytes, the call relocated), manual
//! patch. See `sound/plain_sender.rs` for shadow mode and the self-test.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::plain_sender::{SEND, plain_entry};

const COMMAND: u32 = 0x8;
// ---- Rust version ----

/// `call(callee, command, registers)` for each call the original makes.
pub fn send_command_8(r: &Regs, call: &mut dyn FnMut(usize, u32, &Regs)) {
    call(SEND, COMMAND, r);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, send_command_8)
}
