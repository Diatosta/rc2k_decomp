//! `net::send_chat` (0x4193C8): this player's chat message added to the chat
//! (`net::add_chat_line`, under the received messages' lock) and sent to everyone
//! (`g_netSendTo`, DirectPlay's `Send` through `net::dplay_send`) as '"' and the text, no NUL.
//!
//! In:  `player` (eax), `message` (ebx). The original reads ecx too, only for
//!      `net::add_chat_line`'s first measure (see there): dropped.
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags of `test eax, eax` on
//!      the send's eax are read by no caller: `lq -s 0x4193C8`)
//! Callers: `net::chat_key` (the original through `net::send_chat_regs`, 0x41819B: this
//! inside `pusha` .. `popa`, folded in). Refuses what `net::add_chat_line` refuses.

use crate::errors::Refused;
use crate::net::add_chat_line::add_chat_line;
use crate::net::dplay_send::dplay_send;
use crate::net::store::NET;
use crate::net::{lock_received::lock_received, unlock_received::unlock_received};
use crate::pointers::GamePtr;
use crate::strings::str_append::str_append;

/// The Rust version.
pub fn send_chat(player: u32, message: *const u8) -> Result<(), Refused> {
    lock_received();
    add_chat_line(player, message)?;
    unlock_received();
    let buffer = NET.send_buffer.addr() as u32;
    GamePtr::new(buffer).write(b'"', "the send buffer")?;
    let end =
        crate::pointers::game_address(str_append(message, crate::pointers::host(buffer + 1))?);
    dplay_send(NET.send_to.get(), 0, buffer, end.wrapping_sub(buffer));
    Ok(())
}
