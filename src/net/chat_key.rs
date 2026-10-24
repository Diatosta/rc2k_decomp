//! `net::chat_key` (0x4180E4): a key for the chat message being typed (`net::edit_key`); when
//! that ends the editing and the message is not empty, it is sent (`net::send_chat_regs`,
//! this machine's player) and the input emptied.
//!
//! In:  al = the key
//! Out: as `net::edit_key` leaves them while editing goes on; after it eax = the player,
//!      ebx = the input; no `pusha`
//! Caller: `net::lobby_loop`. Its calls through `part`. Shadow mode runs the original alone
//! (it sends). Self-test: a copy whose calls go to stand-ins, against the Rust version:
//! editing on, ended with a message, ended empty.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::net::lobby::{CHAT_INPUT, PLAYER, a, g, gb, pb};

/// The Rust version: the key `key` for the chat message being typed; once it ends the
/// editing, a message typed is sent and the input cleared. The registers the original
/// leaves (eax the player, ebx the input) are read by no caller (`lobby_loop`).
pub fn chat_key(key: u8) -> Result<(), Refused> {
    if !crate::net::edit_key::edit_key(key)? {
        return Ok(());
    }
    if gb(CHAT_INPUT) != 0 {
        crate::net::send_chat::send_chat(g(PLAYER), a(CHAT_INPUT) as *const u8)?;
        pb(CHAT_INPUT, 0);
    }
    Ok(())
}
