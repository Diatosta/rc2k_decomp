//! `install::key_right` (0x4147C0): key 0x69 on an item (`[item] & 0x2F`): kind 2 the
//! choice after (`install::choice_on`), kind 3 the value up (`install::value_up`).
//!
//! Takes the item. Returns the kind.
//! Caller `install::page_key`, reading zero.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// Whether the key was taken (the original's zero).
pub fn key_right(
    install: &'static crate::install::owner::Install,
    item: ListItem,
) -> Result<bool, Refused> {
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    item_key(install, item, ItemKey::Right, 0)
}
