//! `net::chat_line_locked` (0x4193FF): `net::add_chat_line` with the received messages' lock
//! held (`net::lock_received` .. `net::unlock_received`): a chat message another player sent.
//!
//! In:  `player` (eax), `message` (ebx)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Caller: `net::take_messages`. Refuses what `net::add_chat_line` refuses.

use crate::errors::Refused;
use crate::net::add_chat_line::add_chat_line;
use crate::net::{lock_received::lock_received, unlock_received::unlock_received};

/// The Rust version.
pub fn chat_line_locked(player: u32, message: *const u8) -> Result<(), Refused> {
    lock_received();
    add_chat_line(player, message)?;
    unlock_received();
    Ok(())
}
