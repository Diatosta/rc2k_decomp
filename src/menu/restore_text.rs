//! `menu::restore_text` (0x46D5E3): copies the text entry's backup back into a text field and
//! ends it with a NUL: 0x1F bytes, then a 0 at +0x1F.

use crate::menu::text_entry::{TEXT_LEN, entry as text_entry};

pub fn restore_text(at: impl Into<crate::place::BytePlace>) -> Result<(), crate::errors::BadInput> {
    // The logic is in `rc2k_core::menu::text_entry::Entry::restored`.
    let text: [u8; TEXT_LEN + 1] = text_entry(crate::menu::owner::menus()).restored();
    at.into().write(&text, "a text field")
}
