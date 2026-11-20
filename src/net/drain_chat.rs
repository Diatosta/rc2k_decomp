//! Adds the chat messages the receiving thread took to the chat, in the order they came (a
//! joiner's in the host's numbering). Runs on the game's thread, which owns the fonts a line is
//! measured with. The host also orders the lines its joiners sent: each is added and sent on to
//! every machine, its author's too (`net::relay_chat`), so all show the host's order.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::chat_line_locked::chat_line_locked;
use crate::net::owner::Net;

/// Every waiting message added as `net::chat_line_locked` adds it; the first refusal ends it.
pub(super) fn drain_chat(net: &Net, gfx: &'static Gfx) -> Result<(), Refused> {
    let mut waiting = net.take_waiting_chat();
    // The host's numbers, for lines that arrived together (a stable sort: unnumbered ones, which
    // only the host has, stay as they came).
    waiting.sort_by_key(|line| line.sequence);
    for line in waiting {
        chat_line_locked(net, gfx, line.player, &line.text)?;
        if line.sequence.is_none() {
            let id = net.vars.player_block(line.player)?.id.get();
            crate::net::relay_chat::relay_chat(
                net,
                id,
                crate::net::add_chat_line::text_in(&line.text)?,
            )?;
        }
    }
    Ok(())
}
