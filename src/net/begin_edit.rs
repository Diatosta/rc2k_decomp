//! Starts editing an item's text: notes the item, text, the widest it may be and its font, and
//! keeps the text's 64 bytes so Escape can put them back.

use crate::install::owner::Install;
use crate::net::owner::Net;

/// The texts an edit works on: the settings' machine name, host address and players' count
/// (written into the installer's first path), and the chat's input line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditedText {
    MachineName = 1,
    HostAddress = 2,
    MostPlayers = 3,
    ChatInput = 4,
}

impl EditedText {
    /// The text's buffer.
    pub fn text<'a>(self, net: &'a Net, install: &'a Install) -> &'a dyn crate::cells::Store {
        match self {
            EditedText::MachineName => &net.setup().machine_name,
            EditedText::HostAddress => &net.setup().host_address,
            EditedText::MostPlayers => &install.paths.path_a,
            EditedText::ChatInput => &net.edit().chat_input,
        }
    }

    /// The text `crate::net::owner::net().edit().edit_text` names (refused when none is).
    pub fn edited(net: &Net) -> Result<Self, crate::errors::BadInput> {
        Ok(match net.edit().edit_text.get() {
            1 => EditedText::MachineName,
            2 => EditedText::HostAddress,
            3 => EditedText::MostPlayers,
            4 => EditedText::ChatInput,
            _ => return Err(crate::errors::BadInput("no edited text")),
        })
    }
}
pub fn begin_edit(
    net: &Net,
    install: &Install,
    item: u32,
    text: EditedText,
    width: u32,
    font: crate::gfx::typefaces::FontId,
) -> Result<(), crate::errors::Refused> {
    net.edit().edited.set(item);
    net.edit().edit_text.set(text as u32);
    net.edit().edit_width.set(width);
    net.edit().edit_font.set(font);
    Ok(crate::cells::copy_between(
        text.text(net, install),
        0,
        &install.paths.script_path,
        0,
        0x40,
        "an edited text",
    )?)
}
