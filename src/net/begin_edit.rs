//! `net::begin_edit` (0x416E1A): an item's text starts being edited: the item (`edited`),
//! the text (`edit_text`), the widest it may be (`edit_width`) and the font it is
//! measured with (`edit_font`) noted, and the text's 64 bytes kept (`script_path`) for
//! Escape.
//!
//! Takes `item`, `text`, `width`, `font`.
//! Callers: `net::begin_setting_edit`, `net::begin_chat`.

use crate::net::store::NetVars;

/// The texts an edit works on (`crate::net::owner::net().vars.edit_text`, where the original keeps the address): the
/// settings' machine name, host address and players' count (written into the installer's
/// first path), and the chat's input line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditedText {
    MachineName = 1,
    HostAddress = 2,
    MostPlayers = 3,
    ChatInput = 4,
}

impl EditedText {
    /// The text's buffer.
    pub fn text<'a>(
        self,
        vars: &'a NetVars,
        install: &'a crate::install::owner::Install,
    ) -> &'a dyn crate::pointers::Store {
        match self {
            EditedText::MachineName => &vars.machine_name,
            EditedText::HostAddress => &vars.host_address,
            EditedText::MostPlayers => &install.paths.path_a,
            EditedText::ChatInput => &vars.chat_input,
        }
    }

    /// The text `crate::net::owner::net().vars.edit_text` names (refused when none is).
    pub fn edited(vars: &NetVars) -> Result<Self, crate::errors::BadInput> {
        Ok(match vars.edit_text.get() {
            1 => EditedText::MachineName,
            2 => EditedText::HostAddress,
            3 => EditedText::MostPlayers,
            4 => EditedText::ChatInput,
            _ => return Err(crate::errors::BadInput("no edited text")),
        })
    }
}
pub fn begin_edit(
    vars: &NetVars,
    item: u32,
    text: EditedText,
    width: u32,
    font: crate::gfx::typefaces::FontId,
) -> Result<(), crate::errors::Refused> {
    let install = crate::install::owner::install();
    vars.edited.set(item);
    vars.edit_text.set(text as u32);
    vars.edit_width.set(width);
    vars.edit_font.set(font);
    Ok(crate::pointers::copy_between(
        text.text(vars, install),
        0,
        &install.paths.script_path,
        0,
        0x40,
        "an edited text",
    )?)
}
