//! `sound::send_music_command` (0x409BF6): sends the audio thread command 0xB with the
//! registers as they came, and does not wait.
//!
//! In:  the registers go into the command
//! Out: every register preserved
//! `pusha; push 0xB; call; pop eax; popa`. Callers: `sound::play_music`, `stop_music`, `set_music_volume` (whose Rust versions send it themselves).
//! A call in its first five bytes: private copy (0xB bytes, the call relocated), manual
//! patch. See `sound/plain_sender.rs` for shadow mode and the self-test.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::plain_sender::{SEND, plain_entry};

const COMMAND: u32 = 0xB;
// ---- Rust version ----

/// `call(callee, command, registers)` for each call the original makes.
pub fn send_music_command(r: &Regs, call: &mut dyn FnMut(usize, u32, &Regs)) {
    call(SEND, COMMAND, r);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, send_music_command)
}
