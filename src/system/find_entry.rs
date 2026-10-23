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

pub const ENTRY_LEN: u32 = 8;

// ---- Rust version ----

/// The entry's address, if the key is in the table.
pub unsafe fn find_entry(key: u32, table: u32, count: u32) -> Result<Option<u32>, BadInput> {
    // Converted: `rc2k_core::system::tables::by_key`.
    if count != 0 && table == 0 {
        return Err(BadInput("no table to look in"));
    }
    let found = rc2k_core::system::tables::by_key(unsafe { entries(table, count) }, key);
    Ok(found.map(|n| table.wrapping_add(n as u32 * ENTRY_LEN)))
}

/// The `count` entries from `table`, read as they are asked for.
///
/// # Safety
/// Each entry asked for must be readable.
pub unsafe fn entries(
    table: u32,
    count: u32,
) -> impl Iterator<Item = rc2k_core::system::tables::Entry> {
    (0..count).map(move |n| unsafe {
        ((table.wrapping_add(8 * n)) as *const rc2k_core::system::tables::Entry).read_unaligned()
    })
}
