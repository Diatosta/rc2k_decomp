//! `install::value_down` (0x414881): an item of a value (kind 3) lowered by the key's
//! repeats (`g_listRepeats`), not below its least (`[esi+0Ch]`, signed compare).
//!
//! Returns the value's address, least, new value.
//! Callers `install::key_left`.

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
