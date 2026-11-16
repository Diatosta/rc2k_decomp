//! Key 0x6B on an item: for choices shown upright (kind 0x22) the next choice
//! (`install::choice_on`). Called by `install::page_key`.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::memory::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// Whether the key was taken.
pub fn key_down(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    item: ListItem,
) -> Result<bool, Refused> {
    item_key(install, list_state, item, ItemKey::Down, 0)
}
