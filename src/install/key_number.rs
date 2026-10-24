//! `install::key_number` (0x4147DC): a number key (al 2 .. 0xA) on an item (`[esi] & 0xF`):
//! kind 2 that choice (`install::choice_by_number`), kind 3 that eighth of the value's
//! span (`install::value_by_number`); ecx the key less 2.
//!
//! In:  al the key, esi the item
//! Out: ecx the key less 2, bl the kind; as the helper leaves them (zero set), else the
//!      flags of `cmp bl, 3`
//! Caller `install::page_key`, reading zero.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::key_up::item_key;
use rc2k_core::install::items::ItemKey;

/// `key` the number key (eax); whether it was taken (the original's zero).
pub fn key_number(key: u32, item: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::install::items` (`number_of_key`, `ItemKey::helper`).
    let number = rc2k_core::install::items::number_of_key(key as u8);
    item_key(item, ItemKey::Number, number)
}
