//! `strings::str_append` (0x401F7D): copies a NUL-terminated string to the cursor, without the NUL.
//!
//! In:  eax = source string, ebx = cursor
//! Out: ebx = new cursor

use std::ffi::{CStr, c_char};

// A loop target lies inside the first 5 bytes, so the original runs from a private copy.
// ---- Rust version ----

/// The text at `src` copied to `cursor` without its NUL; the cursor after it.
pub unsafe fn str_append(src: *const u8, cursor: *mut u8) -> *mut u8 {
    unsafe {
        let text = CStr::from_ptr(src as *const c_char).to_bytes();
        std::ptr::copy(text.as_ptr(), cursor, text.len());
        cursor.add(text.len())
    }
}
