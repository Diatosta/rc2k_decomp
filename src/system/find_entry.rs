//! `system::find_entry` (0x40179A): looks a key up in a table of 8-byte entries.
//!
//! Takes the key, the table and the number of entries. Found: the entry whose first dword is
//! the key, and the entries from it to the end. Not found: the table's end and 0 entries. With
//! no entries: nothing changes and it counts as not found.
//! 13 call sites (the video modes, the picture's surface, two more).
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
    let (entries, refused) = entries_checked(table, count);
    let found = rc2k_core::system::tables::by_key(entries, key);
    if let Some(e) = refused.take() {
        return Err(e);
    }
    // A table the caller names by address: its entries are read as bytes.
    Ok(found.map(|n| table.offset(n as u32 * ENTRY_LEN)))
}

/// The table's entries, read inside one allocation each; the walk ends at a refused read,
/// which is left in the cell for the caller to raise before it uses the walk's answer.
pub fn entries_checked(
    table: GamePtr,
    count: u32,
) -> (
    impl Iterator<Item = rc2k_core::system::tables::Entry>,
    std::rc::Rc<std::cell::Cell<Option<BadInput>>>,
) {
    let refused = std::rc::Rc::new(std::cell::Cell::new(None));
    let kept = refused.clone();
    let mut n = 0;
    let walk = std::iter::from_fn(move || {
        if n >= count {
            return None;
        }
        // A caller's table by address, walked as the original does.
        let at = table.offset(ENTRY_LEN.wrapping_mul(n));
        n += 1;
        match at.read::<rc2k_core::system::tables::Entry>("a table's entry") {
            Ok(entry) => Some(entry),
            Err(e) => {
                kept.set(Some(e));
                None
            }
        }
    });
    (walk, refused)
}

/// The table's entries for callers that cannot carry a refusal: a refused read ends the
/// program (`checked_reads::refuse`).
pub fn entries(
    table: GamePtr,
    count: u32,
) -> impl Iterator<Item = rc2k_core::system::tables::Entry> {
    (0..count).map(move |n| {
        // A caller's table by address, walked as the original does.
        table
            .offset(ENTRY_LEN.wrapping_mul(n))
            .read::<rc2k_core::system::tables::Entry>("a table's entry")
            .unwrap_or_else(|e| crate::system::checked_reads::refuse(e))
    })
}
