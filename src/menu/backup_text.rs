//! `menu::backup_text` (0x46D5CC): copies the 0x1F bytes of a text field to the text entry's
//! backup, before the field is edited.
//!
//! In:  edi = the text field
//! Out: every register preserved

use crate::menu::text_entry::{field, with_entry};

// ---- Rust version ----

pub fn backup_text(at: u32) -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::menu::text_entry::Entry::back_up`.
    let text = field(at)?;
    with_entry(|e| e.back_up(&text));
    Ok(())
}
