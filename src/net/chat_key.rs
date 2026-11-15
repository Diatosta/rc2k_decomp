//! A key for the chat message being typed. When the key ends the editing and the message is
//! not empty, it is sent and the input emptied.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::net::owner::net_race;

/// The key `key` for the chat message being typed; once it ends the
/// editing, a message typed is sent and the input cleared.
pub fn chat_key(
    input: &crate::input::owner::Input,
    net: &Net,
    key: u8,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    if !crate::net::edit_key::edit_key(input, net, &net.lobby().edit, key, gfx)? {
        return Ok(());
    }
    if net.lobby().chat_input.get(0) != 0 {
        crate::net::send_chat::send_chat(
            net,
            gfx,
            net_race().player(),
            &net.lobby().chat_input.read(),
        )?;
        net.lobby().chat_input.set(0, 0);
    }
    Ok(())
}
