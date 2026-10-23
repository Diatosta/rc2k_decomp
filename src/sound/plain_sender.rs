//! The game's smallest command senders (stream B, 2026-10-18): one `push n; call
//! sound::send_command` with the registers as they came, and no wait (unlike
//! `sound/senders.rs`'s). What their hooks and self-tests share.
//!
//! Shadow mode runs each original alone (a command must not be sent twice). Self-tests:
//! a copy whose calls go to stand-ins (the one of `send_command` reads the command off
//! the stack), against the Rust version with a part that notes the same.

use crate::framework::Regs;
use crate::sound::queue::{Queue, send_command, sent};

/// `sound::send_command`.
pub const SEND: usize = 0x409C45;

/// A sender's Rust version: it calls `call(callee, command, registers)` for each call.
pub type Sender = fn(&Regs, &mut dyn FnMut(usize, u32, &Regs));

/// A command sent in the game with the registers of `r`.
pub fn send_in_game(command: u32, r: &Regs) {
    unsafe { send_command(&Queue::game(), command, sent(r)) };
}

/// `dispatch_plain` without its hook (no shadow mode; nothing refused).
pub fn plain_entry(r: &mut Regs, rust: Sender) -> Result<(), crate::errors::Refused> {
    rust(r, &mut |callee, command, regs| match callee {
        SEND => send_in_game(command, regs),
        0x409BE0 => crate::sound::send_end_of_turn::send_end_of_turn(),
        // sound::channels_in_use: flags only, which no one reads here.
        _ => {}
    });
    Ok(())
}
