//! `sound::send_end_of_turn` (0x409BE0): tells the audio thread a turn is over
//! (command 0xA).
//!
//! In:  the registers go into the command as they are
//! Out: every register preserved
//! `push eax; push 0xA; call sound::send_command; pop eax; pop eax`. Callers:
//! `sound::race_turn`, `sound::wait_for_commands` (twice, whose Rust version sends it
//! itself), `sound::send_command_7`.
//! A call in its first five bytes: private copy (0xB bytes, the call relocated), manual
//! patch.
//! Shadow mode runs the original alone (a command must not be sent twice).
//! Self-test: a copy whose call goes to a stand-in that notes the registers and the
//! command on the stack, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::sound::queue::{END_OF_TURN, Queue, send_command, sent};
use crate::sound::turn_parts::{Part, turn_entry};

/// `sound::send_command`: its place, and the command it is called with here.
pub const SEND: u32 = 0x409C45;

/// Command 0xA with the registers of `r`, in the game.
pub fn send_end_of_turn_in_game(r: &Regs) {
    unsafe { send_command(&Queue::game(), END_OF_TURN, sent(r)) };
}

// ---- Rust version ----

pub fn send_end_of_turn_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(SEND, r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, send_end_of_turn_in)
}
