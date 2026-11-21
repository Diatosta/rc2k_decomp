//! A key for the chat message being typed. When the key ends the editing and the message is
//! not empty, it is sent and the input emptied.

use crate::errors::Refused;
use crate::net::edit_key::EditOutcome;
use crate::net::owner::Net;

/// The key `key` for the chat message being typed; once it ends the
/// editing, a message typed is sent and the input cleared.
pub(super) fn chat_key(
    strings: &crate::strings::owner::Strings,
    net: &Net,
    key: u8,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    if crate::net::edit_key::edit_key(net, &net.lobby().edit, key, gfx)? != EditOutcome::Done {
        return Ok(());
    }
    let typed = net.lobby().chat_input.read();
    let typed = &typed[..typed.iter().position(|&c| c == 0).unwrap_or(typed.len())];
    if crate::net::kick_command::is_kick(typed) {
        crate::net::kick_command::kick_command(net, strings, gfx, typed)?;
        net.lobby().chat_input.set(0, 0);
        return Ok(());
    }
    if net.lobby().chat_input.get(0) != 0 {
        crate::net::send_chat::send_chat(
            net,
            gfx,
            net.race_slot.player(),
            &net.lobby().chat_input.read(),
        )?;
        net.lobby().chat_input.set(0, 0);
    }
    Ok(())
}
