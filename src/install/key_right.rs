//! Key 0x69 on an item: kind 2 the next choice (`install::choice_on`), kind 3 the value up
//! (`install::value_up`). Called by `install::page_key`.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// Whether the key was taken.
pub fn key_right(
    list_state: &crate::menu::list_state::ListState,
    item: ListItem,
) -> Result<bool, Refused> {
    item_key(list_state, item, ItemKey::Right, 0)
}
