//! Adds the chat messages the receiving thread took to the chat, in the order they came. Runs on
//! the game's thread, which owns the fonts a line is measured with.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::chat_line_locked::chat_line_locked;
use crate::net::owner::Net;

/// Every waiting message added as `net::chat_line_locked` adds it; the first refusal ends it.
pub fn drain_chat(net: &Net, gfx: &'static Gfx) -> Result<(), Refused> {
    for (player, text) in net.take_waiting_chat() {
        chat_line_locked(net, gfx, player, &text)?;
    }
    Ok(())
}
