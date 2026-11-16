//! `sound::send_end_of_turn`: tells the audio thread a turn is over (command 0xA).
//!
//! Sends command 0xA through `sound::send_command`. Callers:
//! `sound::race_turn`, `sound::wait_for_commands`, `sound::send_command_7`.

use crate::sound::queue::{END_OF_TURN, send_command_in};

/// Command 0xA (the audio thread reads none of its words: they go as 0).
pub fn send_end_of_turn(audio: &crate::audio::owner::Audio) {
    send_command_in(audio, END_OF_TURN);
}
