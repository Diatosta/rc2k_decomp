//! `strings::same_text` (0x401EDE): compares two texts without regard to case.
//!
//! In:  eax, ebx = the texts (NUL-terminated)
//! Out: zero set if they are the same; else zero clear, and carry set if the first's
//!      differing character is the lower; every register preserved
//! Character by character, each through `g_lowerCase`. Five callers.

use crate::moved::Place;
use rc2k_core::strings::case::CaseTable;
use std::cmp::Ordering;
use std::ffi::{CStr, c_char};

// ---- Rust version ----

/// The language's lower-case table: out of the image since 2026-10-24 (phase 5; it was
/// `g_lowerCase`, 0x4F0BC8). Until `strings::load_language` sets it up it is the image's:
/// every character itself, but A .. Z and 0xC0 .. 0xDE (not 0xD7) in lower case.
pub static LOWER: Place<[u8; 256]> = Place::new(first_lower());

const fn first_lower() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut c = 0;
    while c < 256 {
        table[c] = match c as u8 {
            b'A'..=b'Z' => c as u8 + 0x20,
            0xC0..=0xDE if c != 0xD7 => c as u8 + 0x20,
            other => other,
        };
        c += 1;
    }
    table
}

/// The language's lower-case table now.
pub fn case_table() -> CaseTable {
    // SAFETY: written only by `load_language` (`set_case_table`), as the image's was.
    CaseTable {
        lower: unsafe { LOWER.ptr().read() },
    }
}

/// A character in lower case (`g_lowerCase[c]`).
pub fn lower(c: u8) -> u8 {
    // SAFETY: as `case_table`.
    unsafe { (*LOWER.ptr())[c as usize] }
}

/// `strings::load_language`'s new table.
pub fn set_case_table(table: &[u8; 256]) {
    // SAFETY: as `case_table`.
    unsafe { LOWER.ptr().write(*table) }
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

#[cfg(test)]
mod tests {
    #[test]
    fn first_table_is_the_games() {
        assert_eq!(
            crate::moved::image_bytes(0x4F0BC8, 256),
            super::case_table().lower
        );
    }
}
