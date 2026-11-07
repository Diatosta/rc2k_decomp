//! `menu::clear_text` (0x467DC9): zeroes the 0x1F bytes of a text field and sets the text
//! entry's length to 0.
//!
//! In:  edi = the text field
//! Out: every register preserved

use crate::menu::owner::Menus;
use crate::menu::text_entry::{Text, put_field, with_entry};

// ---- Rust version ----

pub fn clear_text(
    menus: &Menus,
    at: impl Into<crate::place::Place>,
) -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::menu::text_entry::Entry::clear` (the field is not read).
    let mut text = Text::default();
    with_entry(menus, |e| e.clear(&mut text));
    put_field(at, &text)
}
