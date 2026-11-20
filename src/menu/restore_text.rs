//! `menu::restore_text`: copies the text entry's backup back into a text field and
//! ends it with a NUL: 0x1F bytes, then a 0 after them.

use crate::menu::text_entry::{TEXT_LEN, entry as text_entry};

pub fn restore_text(
    menus: &crate::menu::owner::Menus,
    at: impl Into<crate::memory::place::BytePlace>,
) -> Result<(), crate::errors::BadInput> {
    let text: [u8; TEXT_LEN + 1] = text_entry(menus).restored();
    at.into().write(&text, "a text field")
}
