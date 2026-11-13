//! An item of a value (kind 3) raised by the key's repeats, not above its most (signed
//! compare). Called by `install::key_right`.

use crate::menu::list_handle::ListItem;

pub fn value_up(install: &crate::install::owner::Install, item: ListItem) {
    let value_at = item.value();
    let value = rc2k_core::install::items::value_up(
        value_at.get(),
        install.pages.repeats.get(),
        item.value_most(),
    );
    value_at.set(value as u8);
}
