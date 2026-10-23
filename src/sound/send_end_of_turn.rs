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
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::sound::queue::{END_OF_TURN, Queue, send_command};

// ---- Rust version ----

/// Command 0xA (the audio thread reads none of its registers: they go as 0).
pub fn send_end_of_turn() {
    unsafe { send_command(&Queue::game(), END_OF_TURN, [0; 7]) };
}
