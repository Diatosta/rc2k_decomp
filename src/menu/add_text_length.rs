//! `menu::add_text_length`: adds the length of a NUL-terminated text to the text entry's length.
//! It is used with the length at 0 to set it to the length of a text just restored.
//!
//! Refuses a text with no NUL in its first 4096 bytes.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::text_entry::with_entry;
use rc2k_core::menu::text_entry::text_length;

pub fn add_text_length(
    menus: &Menus,
    text: impl Into<crate::memory::place::BytePlace>,
) -> Result<(), BadInput> {
    let text = text.into();
    let fault = crate::errors::Fault::default();
    let len = text_length(|i| {
        fault.ok(text
            .add(i as u32)
            .read(1, "a name")
            .map(|b| b[0])
            .map_err(|_| BadInput("a name runs past its block")))
    });
    fault.check()?;
    let len = len?;
    with_entry(menus, |e| e.add_length(len));
    Ok(())
}
