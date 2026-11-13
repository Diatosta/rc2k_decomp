//! `net::send_chat` (0x4193C8): this player's chat message added to the chat
//! (`net::add_chat_line`, under the received messages' lock) and sent to everyone
//! (`g_netSendTo`, DirectPlay's `Send` through `net::dplay_send`) as '"' and the text, no NUL.
//!
//! Takes `player` and `message`.
//! Caller: `net::chat_key` (the original through `net::send_chat_regs`, 0x41819B).
//! Refuses what `net::add_chat_line` refuses.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::add_chat_line::add_chat_line;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use crate::net::{lock_received::lock_received, unlock_received::unlock_received};
pub fn send_chat(net: &Net, gfx: &'static Gfx, player: u32, message: &[u8]) -> Result<(), Refused> {
    lock_received(&net.vars);
    add_chat_line(net, gfx, player, message)?;
    unlock_received(&net.vars);
    let text = crate::net::add_chat_line::text_in(message)?;
    let mut line = vec![b'"'];
    line.extend_from_slice(text);
    net.lobby()
        .send_buffer
        .write_at(0, &line, "the send buffer")?;
    dplay_send(&net.vars, net.vars.send_to.get(), &line);
    Ok(())
}
