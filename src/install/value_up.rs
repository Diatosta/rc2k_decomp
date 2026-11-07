//! `install::value_up` (0x41489C): an item of a value (kind 3) raised by the key's repeats
//! (`g_listRepeats`), not above its most (`[esi+0Dh]`, signed compare).
//!
//! In:  esi the item (+8 the value's byte, +0xD the most)
//! Out: edx the value's address, ebx the most, ecx the new value; zero set, carry clear
//! Callers `install::key_right`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::menu::list_handle::ListItem;

pub fn value_up(install: &'static crate::install::owner::Install, item: ListItem) {
    let value_at = item.value();
    // Converted: `rc2k_core::install::items::value_up`.
    let value = rc2k_core::install::items::value_up(
        value_at.get(),
        install.pages.repeats.get(),
        item.value_most(),
    );
    value_at.set(value as u8);
}
