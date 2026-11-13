//! `net::add_chat_line` with the received messages' lock held, for a chat message another
//! player sent. Refuses what `net::add_chat_line` refuses.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::add_chat_line::add_chat_line;
use crate::net::owner::Net;
use crate::net::{lock_received::lock_received, unlock_received::unlock_received};
pub fn chat_line_locked(
    net: &Net,
    gfx: &'static Gfx,
    player: u32,
    message: &[u8],
) -> Result<(), Refused> {
    lock_received(&net.vars);
    add_chat_line(net, gfx, player, message)?;
    unlock_received(&net.vars);
    Ok(())
}
