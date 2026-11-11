//! `menu::backup_text` (0x46D5CC): copies the 0x1F bytes of a text field to the text entry's
//! backup, before the field is edited.

use crate::menu::owner::Menus;
use crate::menu::text_entry::{field, with_entry};

pub fn backup_text(
    menus: &Menus,
    at: impl Into<crate::place::BytePlace>,
) -> Result<(), crate::errors::BadInput> {
    // The logic is in `rc2k_core::menu::text_entry::Entry::back_up`.
    let text = field(at)?;
    with_entry(menus, |e| e.back_up(&text));
    Ok(())
}
