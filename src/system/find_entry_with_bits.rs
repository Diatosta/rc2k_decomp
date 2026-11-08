//! `system::find_entry_with_bits` (0x4017AB): the first of ecx 8-byte entries from ebx
//! whose first dword has all of eax's bits.
//!
//! Takes the bits, the table and the count. Found: the entry and the entries left with it. Not
//! found: past the table and 0 (a count of 0 gives the table as it came).
//! Callers `jb` on the carry: four, the screen modes' tables (`gfx::mark_3d_modes`,
//! 0x4046B7, 0x40471F, 0x404AE6). Refuses a null table with a count (before anything).
//!

// ---- Rust version ----

/// (found, ebx, ecx): the entry found (or the place past the table) and the entries left.
pub fn find_entry_with_bits(
    bits: u32,
    table: crate::pointers::GamePtr,
    count: u32,
) -> (bool, crate::pointers::GamePtr, u32) {
    // Converted: `rc2k_core::system::tables::with_bits`.
    use rc2k_core::system::tables::with_bits;
    let found = with_bits(crate::system::find_entry::entries(table, count), bits);
    // The entry's place in a caller's table by address (the original returns the pointer).
    match found {
        Some(n) => (true, table.offset(8 * n as u32), count - n as u32),
        None => (false, table.offset(count.wrapping_mul(8)), 0),
    }
}
