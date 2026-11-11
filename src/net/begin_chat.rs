//! `net::begin_chat` (0x4180C5): Enter on the lobby's chat item starts a message: the input
//! (`chat_input`) emptied and edited (`net::begin_edit`, the item `g_listChoice`, width
//! 0x30C, measured with `typefaces::CHAT_LINE`).
//!
//! Returns the item, input, 0x30C, the chat font.
//! Caller: `net::lobby_loop`.

use crate::install::owner::Install;
use crate::net::owner::Net;

const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CHAT_LINE;

/// Starts the chat line's edit (item, input, width 0x30C and font as the original passes them).
pub fn begin_chat(net: &Net, install: &Install) -> Result<(), crate::errors::Refused> {
    net.edit().chat_input.set(0, 0);
    crate::net::begin_edit::begin_edit(
        net,
        install,
        install.pages.choice.get(),
        crate::net::begin_edit::EditedText::ChatInput,
        0x30C,
        CHAT_FONT,
    )
}
