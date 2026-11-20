//! Starts editing an item's text: notes the item, text, the widest it may be and its font, and
//! keeps the text's 64 bytes so Escape can put them back.

use crate::net::owner::Net;
use crate::net::store::Editing;
use rc2k_core::net::editing;

/// The texts an edit works on: the settings' machine name, host address and players' count,
/// and the chat's input line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditedText {
    MachineName = 1,
    HostAddress = 2,
    MostPlayers = 3,
    ChatInput = 4,
}

impl EditedText {
    /// The text's buffer.
    pub fn text(self, net: &Net) -> &dyn crate::memory::cells::Store {
        match self {
            EditedText::MachineName => &net.setup().machine_name,
            EditedText::HostAddress => &net.setup().host_address,
            EditedText::MostPlayers => &net.settings().most_players_text,
            EditedText::ChatInput => &net.lobby().chat_input,
        }
    }
}

/// Which text an edit works on, kept in a record of cells (none: nothing is edited).
#[repr(transparent)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct EditedTextCell(crate::memory::session::Dword);

impl EditedTextCell {
    pub const fn new() -> Self {
        Self(crate::memory::session::Dword::new(0))
    }

    pub fn get(&self) -> Option<EditedText> {
        match self.0.get() {
            1 => Some(EditedText::MachineName),
            2 => Some(EditedText::HostAddress),
            3 => Some(EditedText::MostPlayers),
            4 => Some(EditedText::ChatInput),
            _ => None,
        }
    }

    pub fn set(&self, text: EditedText) {
        self.0.set(text as u32);
    }
}

/// Starts the edit `edit` of `text` for item `item`.
pub fn begin_edit(
    net: &Net,
    edit: &Editing,
    item: u32,
    text: EditedText,
    width: u32,
    font: crate::gfx::typefaces::FontId,
) -> Result<(), crate::errors::Refused> {
    edit.edited.set(item);
    edit.edit_text.set(text);
    edit.edit_width.set(width);
    edit.edit_font.set(font);
    Ok(crate::memory::cells::copy_between(
        text.text(net),
        0,
        &edit.text_before_edit,
        0,
        editing::TEXT_LEN,
        "an edited text",
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::store::NetMain;

    /// The settings and the lobby each keep their own edit: starting one leaves the other
    /// untouched, and Escape's copy comes from the text edited.
    #[test]
    fn each_screen_edits_apart() {
        let main = NetMain::new();
        main.settings.edit.edited.set(u32::MAX);
        main.lobby.edit.edited.set(u32::MAX);
        main.setup.machine_name.set(0, b'A');
        main.settings.edit.edit_text.set(EditedText::MachineName);
        assert_eq!(
            main.settings.edit.edit_text.get(),
            Some(EditedText::MachineName)
        );
        assert_eq!(main.lobby.edit.edit_text.get(), None);
        assert_eq!(main.lobby.edit.edited.get(), u32::MAX);
        assert_eq!(main.settings.edit.text_before_edit.get(0), 0);
    }
}
