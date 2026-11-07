//! `install::key_left` (0x4147A4): key 0x68 on an item (`[esi] & 0x2F`): kind 2 the choice
//! before (`install::choice_back`), kind 3 the value down (`install::value_down`).
//!
//! Takes the item. Returns the kind.
//! Caller `install::page_key`, reading zero.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// Whether the key was taken (the original's zero).
pub fn key_left(
    install: &'static crate::install::owner::Install,
    item: ListItem,
) -> Result<bool, Refused> {
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    item_key(install, item, ItemKey::Left, 0)
}
