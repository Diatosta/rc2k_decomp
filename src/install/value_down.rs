//! `install::value_down` (0x414881): an item of a value (kind 3) lowered by the key's
//! repeats (`g_listRepeats`), not below its least (`[esi+0Ch]`, signed compare).
//!
//! In:  esi the item (+8 the value's byte, +0xC the least)
//! Out: edx the value's address, ebx the least, ecx the new value; zero set, carry clear
//! Callers `install::key_left`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::menu::list_handle::ListItem;

pub fn value_down(install: &'static crate::install::owner::Install, item: ListItem) {
    let value_at = item.value();
    // Converted: `rc2k_core::install::items::value_down`.
    let value = rc2k_core::install::items::value_down(
        value_at.get(),
        install.pages.repeats.get(),
        item.value_least(),
    );
    value_at.set(value as u8);
}
