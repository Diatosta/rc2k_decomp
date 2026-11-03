//! `install::key_up` (0x414784): key 0x66 on an item: for choices shown upright (kind
//! 0x22 of `[esi] & 0x2F`) the choice before (`install::choice_back`).
//!
//! In:  esi the item
//! Out: bl the kind (`[esi] & 0x2F`); with choices as `choice_back` leaves them (zero set),
//!      else the flags of `cmp bl, 22h` (zero clear)
//! Caller `install::page_key`, reading zero.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::menu::list_handle::ListItem;
use rc2k_core::install::items::{Helper, ItemKey};

/// A key on `item`: the helper the key has for the item's kind, `number` for the number
/// keys' helpers; the original's zero after it (the helpers' set, else that of the last
/// compare of the kind).
pub fn item_key(item: ListItem, key: ItemKey, number: u32) -> Result<bool, Refused> {
    let kind = item.first() & key.kind_mask();
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    use crate::install as i;
    match key.helper(kind) {
        Some(Helper::ChoiceBack) => i::choice_back::choice_back(item)?,
        Some(Helper::ChoiceOn) => i::choice_on::choice_on(item)?,
        Some(Helper::ChoiceByNumber) => i::choice_by_number::choice_by_number(item, number),
        Some(Helper::ValueDown) => i::value_down::value_down(item),
        Some(Helper::ValueUp) => i::value_up::value_up(item),
        Some(Helper::ValueByNumber) => i::value_by_number::value_by_number(item, number)?,
        None => return Ok(kind == key.last_compared()),
    }
    Ok(true)
}

/// Whether the key was taken (the original's zero).
pub fn key_up(item: ListItem) -> Result<bool, Refused> {
    item_key(item, ItemKey::Up, 0)
}
