//! `strings::str_append_indexed` (0x401F8E): appends string number `index` of a packed table
//! of NUL-terminated strings.
//!
//! In:  eax = table, ecx = index, ebx = cursor
//! Out: ebx = new cursor
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

use crate::errors::BadInput;
use crate::pointers::GamePtr;

// `detour` refuses to patch this function, so the jump is written by hand and the original
// runs from a private copy (its call at +0x0D relocated).
// ---- Rust version ----

/// String `index` of the list at `table` (converted: `rc2k_core::strings::table::nth_string`, on
/// the list as far as the original reads it), copied out; the list must run that far inside
/// its block.
pub fn indexed_text(table: *const u8, index: i32) -> Result<Vec<u8>, BadInput> {
    let table = crate::pointers::game_address(table);
    let mut length = 0;
    for _ in 0..=index.max(0) {
        length +=
            crate::strings::text_length::nul_at(table.wrapping_add(length), "a string list")? + 1;
    }
    let list = GamePtr::new(table).read_bytes(length as usize, "a string list")?;
    Ok(rc2k_core::strings::table::nth_string(&list, index).to_vec())
}

/// String `index` of the list at `table` appended at `cursor`; the cursor after it.
pub fn str_append_indexed(
    table: *const u8,
    index: i32,
    cursor: *mut u8,
) -> Result<*mut u8, BadInput> {
    let text = indexed_text(table, index)?;
    GamePtr::new(crate::pointers::game_address(cursor))
        .write_bytes(&text, "a string list\'s target")?;
    Ok(cursor.wrapping_add(text.len()))
}
