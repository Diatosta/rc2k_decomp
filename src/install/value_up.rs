//! `install::value_up` (0x41489C): an item of a value (kind 3) raised by the key's repeats
//! (`g_listRepeats`), not above its most (`[item+0Dh]`, signed compare).
//!
//! Returns the value's address, most, new value.
//! Callers `install::key_right`.

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
