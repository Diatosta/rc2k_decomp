//! Sends this player's chat message. The host adds it to its chat and sends it on to everyone
//! with its number (`net::relay_chat`); a joiner sends it to the host alone as a chat message (`"`
//! and the text, no NUL) and shows it when the host's order brings it back, so the same lines
//! show in the same order on every machine. Refuses what `net::add_chat_line` refuses.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::add_chat_line::{add_chat_line, text_in};
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use rc2k_core::net::message::{CHAT_TEXT_MOST, Message};
pub(super) fn send_chat(
    net: &Net,
    gfx: &'static Gfx,
    player: u32,
    message: &[u8],
) -> Result<(), Refused> {
    crate::net::drain_chat::drain_chat(net, gfx)?;
    let text = text_in(message)?;
    let text = &text[..text.len().min(CHAT_TEXT_MOST)];
    if net.vars.own.state().is_host() {
        {
            let _received = net.lock_received();
            add_chat_line(net, gfx, player, message)?;
        }
        if !text.is_empty() {
            crate::net::relay_chat::relay_chat(net, net.link.player_id(), text)?;
        }
        return Ok(());
    }
    let line = Message::Chat(text.to_vec()).encode();
    net.lobby()
        .send_buffer
        .write_at(0, &line, "the send buffer")?;
    let host = crate::net::dp_objects::host_player(net.link.dplay()).unwrap_or(net.link.send_to());
    dplay_send(&net.link, host, &line);
    Ok(())
}
