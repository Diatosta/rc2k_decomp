//! Enter on the lobby's chat item starts a message: the input is emptied and edited with
//! `net::begin_edit`. Returns the item, input, width and font the edit uses.

use crate::net::owner::Net;

const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CHAT_LINE;

/// Starts the chat line's edit (item, input, width 0x30C and the chat font).
pub fn begin_chat(
    net: &Net,
    list_state: &crate::menu::list_state::ListState,
) -> Result<(), crate::errors::Refused> {
    net.edit().chat_input.set(0, 0);
    crate::net::begin_edit::begin_edit(
        net,
        list_state.choice.get(),
        crate::net::begin_edit::EditedText::ChatInput,
        0x30C,
        CHAT_FONT,
    )
}
