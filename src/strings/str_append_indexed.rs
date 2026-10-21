//! `strings::str_append_indexed` (0x401F8E): appends string number `index` of a packed table
//! of NUL-terminated strings.
//!
//! In:  eax = table, ecx = index, ebx = cursor
//! Out: ebx = new cursor
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

use std::ffi::{CStr, c_char};

// `detour` refuses to patch this function, so the jump is written by hand and the original
// runs from a private copy (its call at +0x0D relocated).
// ---- Rust version ----

/// String `index` of the list at `table` (converted: `rc2k_core::strings::table::nth_string`, on
/// the list as far as the original reads it).
pub unsafe fn indexed_text<'a>(table: *const u8, index: i32) -> &'a [u8] {
    unsafe {
        let mut length = 0;
        for _ in 0..=index.max(0) {
            length += CStr::from_ptr(table.add(length) as *const c_char)
                .to_bytes_with_nul()
                .len();
        }
        rc2k_core::strings::table::nth_string(std::slice::from_raw_parts(table, length), index)
    }
}

/// String `index` of the list at `table` appended at `cursor`; the cursor after it.
pub unsafe fn str_append_indexed(table: *const u8, index: i32, cursor: *mut u8) -> *mut u8 {
    unsafe {
        let text = indexed_text(table, index);
        std::ptr::copy(text.as_ptr(), cursor, text.len());
        cursor.add(text.len())
    }
}
