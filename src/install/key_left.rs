//! Key 0x68 on an item: kind 2 the previous choice (`install::choice_back`), kind 3 the value
//! down (`install::value_down`). Called by `install::page_key`.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::memory::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// Whether the key was taken.
pub fn key_left(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    item: ListItem,
) -> Result<bool, Refused> {
    item_key(install, list_state, item, ItemKey::Left, 0)
}
