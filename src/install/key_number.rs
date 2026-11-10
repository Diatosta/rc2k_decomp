//! `install::key_number` (0x4147DC): a number key (al 2 .. 0xA) on an item (`[esi] & 0xF`):
//! kind 2 that choice (`install::choice_by_number`), kind 3 that eighth of the value's
//! span (`install::value_by_number`); ecx the key less 2.
//!
//! Takes the key, item. Returns the key less 2, kind.
//! Caller `install::page_key`, reading zero.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// `key` the number key (eax); whether it was taken (the original's zero).
pub fn key_number(
    install: &'static crate::install::owner::Install,
    key: u32,
    item: ListItem,
) -> Result<bool, Refused> {
    // Converted: `rc2k_core::install::items` (`number_of_key`, `ItemKey::helper`).
    let number = rc2k_core::install::items::number_of_key(key as u8);
    item_key(install, item, ItemKey::Number, number)
}
