//! `system::find_entry` (0x40179A): looks a key up in a table of 8-byte entries.
//!
//! In:  eax = the key, ebx = the table, ecx = the number of entries
//! Out: found: ebx = the entry whose first dword is the key, ecx = the entries from it to
//!      the end, carry clear, zero set. Not found: ebx = the table's end, ecx = 0, carry
//!      and zero set. With no entries: nothing changes, carry set, zero clear.
//!      Everything else preserved.
//! 13 call sites (the video modes, the picture's surface, two more).
//! Manual patch with a private copy (0x11 bytes, nothing to relocate): a conditional jump
//! sits in the first bytes.
//! Refuses a null table with entries. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::pointers::GamePtr;

pub const ENTRY_LEN: u32 = 8;

// ---- Rust version ----

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
        table
            .offset(ENTRY_LEN.wrapping_mul(n))
            .read::<rc2k_core::system::tables::Entry>("a table's entry")
            .unwrap_or_else(|e| crate::system::checked_reads::refuse(e))
    })
}
