//! `install::key_down` (0x414794): key 0x6B on an item: for choices shown upright (kind
//! 0x22 of `[item] & 0x2F`) the choice after (`install::choice_on`).
//!
//! Takes the item. Returns the kind (`[item] & 0x2F`).
//! Caller `install::page_key`, reading zero.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// Whether the key was taken (the original's zero).
pub fn key_down(install: &crate::install::owner::Install, item: ListItem) -> Result<bool, Refused> {
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    item_key(install, item, ItemKey::Down, 0)
}
