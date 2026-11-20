//! The host's ordering of the lobby chat: a line it has added to its own chat is numbered and
//! sent to every other machine's player, the author's included. TCP keeps each machine's lines
//! in the order sent, so every machine ends with the host's order; the number lets a machine
//! that took several at once keep them in it.

use crate::errors::Refused;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use rc2k_core::net::message::Message;

/// `text` (without its NUL), written by the player with DirectPlay id `author`, sent to every
/// player but this machine's own.
pub(super) fn relay_chat(net: &Net, author: u32, text: &[u8]) -> Result<(), Refused> {
    let line = Message::ChatRelayed {
        sequence: net.next_chat_sequence(),
        author,
        text: text.to_vec(),
    }
    .encode();
    for n in 0..net.race_slot.players() {
        let id = net.vars.player_block(n)?.id.get();
        if id != net.link.player_id() {
            dplay_send(&net.link, id, &line);
        }
    }
    Ok(())
}
