//! `strings::same_text` (0x401EDE): compares two texts without regard to case.
//!
//! In:  eax, ebx = the texts (NUL-terminated)
//! Out: zero set if they are the same; else zero clear, and carry set if the first's
//!      differing character is the lower; every register preserved
//! Character by character, each through `g_lowerCase`. Five callers.

use crate::bridge::Global;
use rc2k_core::strings::case::CaseTable;
use std::cmp::Ordering;
use std::ffi::{CStr, c_char};

// ---- Rust version ----

/// The language's lower-case table (`g_lowerCase`, set up by `strings::load_language`).
pub fn case_table() -> CaseTable {
    static LOWER: Global<[u8; 256]> = Global::at(0x4F_0BC8);
    CaseTable { lower: LOWER.get() }
}

/// How the first text's first differing character stands to the second's (converted:
/// `rc2k_core::strings::case::CaseTable::compare`).
pub unsafe fn compare_texts(a: *const u8, b: *const u8) -> Ordering {
    let (a, b) = unsafe {
        (
            CStr::from_ptr(a as *const c_char),
            CStr::from_ptr(b as *const c_char),
        )
    };
    case_table().compare(a.to_bytes(), b.to_bytes())
}

pub unsafe fn same_text(a: *const u8, b: *const u8) -> bool {
    unsafe { compare_texts(a, b) == Ordering::Equal }
}
