//! `net::add_chat_line` with the received messages' lock held, for a chat message another
//! player sent. Refuses what `net::add_chat_line` refuses.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::add_chat_line::add_chat_line;
use crate::net::owner::Net;
pub fn chat_line_locked(
    net: &Net,
    gfx: &'static Gfx,
    player: u32,
    message: &[u8],
) -> Result<(), Refused> {
    let _received = net.lock_received();
    add_chat_line(net, gfx, player, message)?;
    Ok(())
}
