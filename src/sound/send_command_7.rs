//! `sound::send_command_7` (0x409B8C): command 7 to the audio thread, then the end of
//! the turn, then `sound::channels_in_use` (whose flags `popa` / `retn` throw away).
//!
//! In:  the registers go into the commands
//! Out: every register preserved (`pusha` / `popa`)
//! Command 7 with the registers as they came; command 0xA (`sound::send_end_of_turn`)
//! with eax 7 (the `pop eax` of the push), the rest as they came. Callers:
//! `race::reset_race_sounds` (0x40D37E) and 0x409BA9.
//! A call in its first five bytes: private copy (0x15 bytes, the three calls relocated),
//! manual patch. See `sound/plain_sender.rs` for shadow mode and the self-test.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::sound::plain_sender::{SEND, plain_entry};

const END_OF_TURN: usize = 0x409BE0;
const CHANNELS: usize = 0x409BA1;
// ---- Rust version ----

pub fn send_command_7(r: &Regs, call: &mut dyn FnMut(usize, u32, &Regs)) {
    call(SEND, 7, r);
    let after = Regs { eax: 7, ..*r };
    call(END_OF_TURN, 0, &after);
    call(CHANNELS, 0, &after);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, send_command_7)
}
