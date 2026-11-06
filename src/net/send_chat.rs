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
use crate::net::owner::net;
use crate::net::{lock_received::lock_received, unlock_received::unlock_received};
use crate::pointers::GamePtr;

/// The Rust version.
pub fn send_chat(player: u32, message: GamePtr) -> Result<(), Refused> {
    let net = net();
    lock_received();
    add_chat_line(player, message)?;
    unlock_received();
    let text = crate::files::cstring(message, "a chat line")?;
    let mut line = vec![b'"'];
    line.extend_from_slice(&text);
    net.vars.send_buffer.write_at(0, &line, "the send buffer")?;
    dplay_send(net.vars.send_to.get(), &line);
    Ok(())
}
