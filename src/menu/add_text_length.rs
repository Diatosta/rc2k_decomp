//! `menu::add_text_length` (0x46F4CE): adds the length of the NUL-terminated text at edi to
//! the text entry's length. Its callers use it with the length at 0, to set it to the length
//! of a text just restored.
//!
//! Refuses a text with no NUL in its first 0x1000 bytes.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::text_entry::with_entry;
use rc2k_core::menu::text_entry::text_length;

pub fn add_text_length(
    menus: &Menus,
    text: impl Into<crate::place::BytePlace>,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::text_entry::{text_length, Entry::add_length}`.
    let text = text.into();
    let fault = crate::menu::event_flow::Fault::default();
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
