//! `sound::send_end_of_turn` (0x409BE0): tells the audio thread a turn is over
//! (command 0xA).
//!
//! `push eax; push 0xA; call sound::send_command; pop eax; pop eax`. Callers:
//! `sound::race_turn`, `sound::wait_for_commands` (twice, whose Rust version sends it
//! itself), `sound::send_command_7`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::sound::queue::{END_OF_TURN, send_command};

/// Command 0xA (the audio thread reads none of its registers: they go as 0).
pub fn send_end_of_turn() {
    send_command(END_OF_TURN, [0; 7]);
}
