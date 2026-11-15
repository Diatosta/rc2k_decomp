//! A number key (2 .. 0xA) on an item: kind 2 that choice (`install::choice_by_number`), kind 3
//! that eighth of the value's span (`install::value_by_number`); the number is the key less 2.
//! Called by `install::page_key`.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use crate::list_handle::ListItem;
use rc2k_core::install::items::ItemKey;

/// `key` the number key; whether it was taken.
pub fn key_number(
    install: &'static crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    key: u32,
    item: ListItem,
) -> Result<bool, Refused> {
    let number = rc2k_core::install::items::number_of_key(key as u8);
    item_key(install, list_state, item, ItemKey::Number, number)
}
