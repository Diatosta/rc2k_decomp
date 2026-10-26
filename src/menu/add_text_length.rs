//! `menu::add_text_length` (0x46F4CE): adds the length of the NUL-terminated text at edi to
//! the text entry's length. Its callers use it with the length at 0, to set it to the length
//! of a text just restored.
//!
//! In:  edi = text
//! Out: ecx = ecx + 1 (an `inc ecx` after the `popa`); everything else preserved
//! Refuses a text with no NUL in its first 0x1000 bytes.

use crate::errors::BadInput;
use crate::menu::text_entry::with_entry;
use rc2k_core::menu::text_entry::text_length;
use rc2k_core::strings::bytes::Bytes;

// ---- Rust version ----

pub fn add_text_length(text: u32) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::text_entry::{text_length, Entry::add_length}`.
    let bytes = crate::pointers::GamePtr::new(text).bytes("a name")?;
    let fault = crate::menu::event_flow::Fault::default();
    let len =
        text_length(|i| fault.ok(bytes.byte(i).ok_or(BadInput("a name runs past its block"))));
    fault.check()?;
    let len = len?;
    with_entry(|e| e.add_length(len));
    Ok(())
}
