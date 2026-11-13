//! `menu::clear_text`: zeroes the 0x1F bytes of a text field and sets the text
//! entry's length to 0.

use crate::menu::owner::Menus;
use crate::menu::text_entry::{Text, put_field, with_entry};

pub fn clear_text(
    menus: &Menus,
    at: impl Into<crate::place::BytePlace>,
) -> Result<(), crate::errors::BadInput> {
    let mut text = Text::default();
    with_entry(menus, |e| e.clear(&mut text));
    put_field(at, &text)
}
