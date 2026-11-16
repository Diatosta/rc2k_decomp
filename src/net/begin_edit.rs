//! Starts editing an item's text: notes the item, text, the widest it may be and its font, and
//! keeps the text's 64 bytes so Escape can put them back.

use crate::net::owner::Net;
use crate::net::store::Editing;

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

    /// The text `edit` names (refused when none is).
    pub fn edited(edit: &Editing) -> Result<Self, crate::errors::BadInput> {
        Ok(match edit.edit_text.get() {
            1 => EditedText::MachineName,
            2 => EditedText::HostAddress,
            3 => EditedText::MostPlayers,
            4 => EditedText::ChatInput,
            _ => return Err(crate::errors::BadInput("no edited text")),
        })
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
    edit.edit_text.set(text as u32);
    edit.edit_width.set(width);
    edit.edit_font.set(font);
    Ok(crate::memory::cells::copy_between(
        text.text(net),
        0,
        &edit.text_before_edit,
        0,
        0x40,
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
        main.settings
            .edit
            .edit_text
            .set(EditedText::MachineName as u32);
        assert_eq!(
            EditedText::edited(&main.settings.edit).unwrap(),
            EditedText::MachineName
        );
        assert!(EditedText::edited(&main.lobby.edit).is_err());
        assert_eq!(main.lobby.edit.edited.get(), u32::MAX);
        assert_eq!(main.settings.edit.text_before_edit.get(0), 0);
    }
}
