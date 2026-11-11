//! `system::find_entry` (0x40179A): looks a key up in a table of 8-byte entries.
//!
//! Takes the key, the table and the number of entries. Found: the entry whose first dword is
//! the key, and the entries from it to the end. Not found: the table's end and 0 entries. With
//! no entries: nothing changes and it counts as not found.
//! Used for the card's texture sizes.
//! Refuses a null table with entries. The name is a reading.
//!

use crate::errors::BadInput;
use crate::pointers::GamePtr;

pub const ENTRY_LEN: u32 = 8;

/// The entry, if the key is in the table.
pub fn find_entry(key: u32, table: GamePtr, count: u32) -> Result<Option<GamePtr>, BadInput> {
    // Converted: `rc2k_core::system::tables::by_key`.
    if count != 0 && table.is_null() {
        return Err(BadInput("no table to look in"));
    }
    let refused = std::cell::Cell::new(None);
    let mut n = 0;
    let walk = std::iter::from_fn(|| {
        if n >= count {
            return None;
        }
        // A caller's table by address, walked as the original does.
        let at = table.offset(ENTRY_LEN.wrapping_mul(n));
        n += 1;
        at.read::<rc2k_core::system::tables::Entry>("a table's entry")
            .map_err(|e| refused.set(Some(e)))
            .ok()
    });
    let found = rc2k_core::system::tables::by_key(walk, key);
    if let Some(e) = refused.take() {
        return Err(e);
    }
    // A table the caller names by address: its entries are read as bytes.
    Ok(found.map(|n| table.offset(n as u32 * ENTRY_LEN)))
}
