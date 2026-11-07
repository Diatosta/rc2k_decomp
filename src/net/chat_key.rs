//! `net::chat_key` (0x4180E4): a key for the chat message being typed (`net::edit_key`); when
//! that ends the editing and the message is not empty, it is sent (`net::send_chat_regs`,
//! this machine's player) and the input emptied.
//!
//! Takes the key.
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::net::owner::Net;

/// The Rust version: the key `key` for the chat message being typed; once it ends the
/// editing, a message typed is sent and the input cleared. The registers the original
/// leaves (eax the player, ebx the input) are read by no caller (`lobby_loop`).
pub fn chat_key(net: &Net, key: u8) -> Result<(), Refused> {
    let vars = &net.vars;
    if !crate::net::edit_key::edit_key(net, key)? {
        return Ok(());
    }
    if vars.chat_input.get(0) != 0 {
        crate::net::send_chat::send_chat(net, vars.player.get(), &vars.chat_input.read())?;
        vars.chat_input.set(0, 0);
    }
    Ok(())
}
