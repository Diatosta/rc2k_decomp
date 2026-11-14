//! Adds this player's chat message to the chat and sends it to everyone as a chat message (`"` and
//! the text, no NUL). Refuses what `net::add_chat_line` refuses.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::net::add_chat_line::add_chat_line;
use crate::net::dplay_send::dplay_send;
use crate::net::owner::Net;
use rc2k_core::net::message::Message;
pub fn send_chat(net: &Net, gfx: &'static Gfx, player: u32, message: &[u8]) -> Result<(), Refused> {
    {
        let _received = net.lock_received();
        add_chat_line(net, gfx, player, message)?;
    }
    let text = crate::net::add_chat_line::text_in(message)?;
    let line = Message::Chat(text.to_vec()).encode();
    net.lobby()
        .send_buffer
        .write_at(0, &line, "the send buffer")?;
    dplay_send(&net.link, net.link.send_to(), &line);
    Ok(())
}
