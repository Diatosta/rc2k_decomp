//! An item of a value (kind 3) lowered by the key's repeats, not below its least (signed
//! compare). Called by `install::key_left`.

use crate::menu::list_handle::ListItem;

pub fn value_down(list_state: &crate::menu::list_state::ListState, item: ListItem) {
    let value_at = item.value();
    let value = rc2k_core::install::items::value_down(
        value_at.get(),
        list_state.repeats.get(),
        item.value_least(),
    );
    value_at.set(value as u8);
}
