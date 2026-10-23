//! `menu::add_text_length` (0x46F4CE): adds the length of the NUL-terminated text at edi to
//! the text entry's length. Its callers use it with the length at 0, to set it to the length
//! of a text just restored.
//!
//! In:  edi = text
//! Out: ecx = ecx + 1 (an `inc ecx` after the `popa`); everything else preserved
//! Refuses a text with no NUL in its first 0x1000 bytes.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::menu::text_entry::with_entry;
use rc2k_core::menu::text_entry::text_length;

// ---- Rust version ----

pub unsafe fn add_text_length(text: *const u8) -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::text_entry::{text_length, Entry::add_length}`.
    let len = text_length(|i| unsafe { text.add(i).read() })?;
    with_entry(|e| e.add_length(len));
    Ok(())
}
