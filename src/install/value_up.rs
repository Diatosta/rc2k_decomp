//! An item of a value (kind 3) raised by the key's repeats, not above its most (signed compare).

use crate::memory::list_handle::ListItem;

pub fn value_up(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    item: ListItem,
) {
    let value_at = item.value(install);
    let value = rc2k_core::install::items::value_up(
        value_at.get(),
        list_state.repeats.get(),
        item.value_most(),
    );
    value_at.set(value as u8);
}
