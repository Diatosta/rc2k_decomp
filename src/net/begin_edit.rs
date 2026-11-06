//! `net::begin_edit` (0x416E1A): an item's text starts being edited: the item (`dword_52A4B8`),
//! the text (`dword_52A4B4`), the widest it may be (`dword_52A4BC`) and the font it is
//! measured with (`dword_52A4C0`) noted, and the text's 64 bytes kept (`byte_95EAF8`) for
//! Escape.
//!
//! In:  `item` (eax), `text` (ebx), `width` (ecx), `font` (edx)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Callers: `net::begin_setting_edit`, `net::begin_chat`.

use crate::net::lobby::movsb;

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
    /// The text, bounded by its buffer.
    pub fn place(self) -> crate::pointers::GamePtr {
        use crate::pointers::GamePtr;
        match self {
            EditedText::MachineName => GamePtr::to(&crate::net::owner::net().vars.machine_name),
            EditedText::HostAddress => GamePtr::to(&crate::net::owner::net().vars.host_address),
            EditedText::MostPlayers => GamePtr::to(&crate::install::owner::install().paths.path_a),
            EditedText::ChatInput => GamePtr::to(&crate::net::owner::net().vars.chat_input),
        }
    }

    /// The text `crate::net::owner::net().vars.edit_text` names (refused when none is).
    pub fn edited() -> Result<Self, crate::errors::BadInput> {
        Ok(match crate::net::owner::net().vars.edit_text.get() {
            1 => EditedText::MachineName,
            2 => EditedText::HostAddress,
            3 => EditedText::MostPlayers,
            4 => EditedText::ChatInput,
            _ => return Err(crate::errors::BadInput("no edited text")),
        })
    }
}

/// The Rust version.
pub fn begin_edit(
    item: u32,
    text: EditedText,
    width: u32,
    font: crate::gfx::typefaces::FontId,
) -> Result<(), crate::errors::Refused> {
    crate::net::owner::net().vars.edited.set(item);
    crate::net::owner::net().vars.edit_text.set(text as u32);
    crate::net::owner::net().vars.edit_width.set(width);
    crate::net::owner::net().vars.edit_font.set(font);
    movsb(
        &crate::install::owner::install().paths.script_path,
        text.place(),
        0x40,
    )
}
