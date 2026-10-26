//! `strings::append_string_by_id` (0x401FA3): appends the text for a string id.
//!
//! In:  eax = id, ebx = cursor
//! Out: ebx = new cursor; eax = the string pointer (pointer-table branches) or the id itself
//!      (string-table branch)
//! Part of the original lives in a separate chunk at 0x402010.

use crate::strings::expand_entry::expand_entry;
use crate::strings::string_table::{blob, ptr_high, ptr_low};
use rc2k_core::strings::table::Source;
use std::ffi::{CStr, c_char};
use std::slice::from_raw_parts;

// ---- Rust version ----

/// `emit` receives each run of bytes to append. Returns the `eax` the original leaves behind.
pub unsafe fn append_string_by_id(id: u32, emit: &mut impl FnMut(*const u8, usize)) -> u32 {
    unsafe {
        // Converted: `rc2k_core::strings::table::Source`.
        let direct = match Source::of(id, || !ptr_low(id).is_null()) {
            Source::Made => Some(ptr_high(id)),
            Source::Pointer => Some(ptr_low(id)),
            Source::Table => None,
        };
        match direct {
            Some(p) => {
                let len = CStr::from_ptr(p as *const c_char).to_bytes().len();
                emit(p, len);
                p as usize as u32
            }
            None => {
                expand_entry(blob(), id, emit);
                id
            }
        }
    }
}

/// Appends the text of string `id` to `out`.
pub unsafe fn push_string(out: &mut Vec<u8>, id: u32) {
    unsafe { append_string_by_id(id, &mut |p, n| out.extend_from_slice(from_raw_parts(p, n))) };
}
