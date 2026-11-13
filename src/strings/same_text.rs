//! `strings::same_text` (0x401EDE): compares two texts without regard to case.
//!
//! Takes the texts (NUL-terminated). Returns whether they are the same, and if not whether the
//! first's differing character is the lower.
//! Character by character, each through `g_lowerCase`. Five callers.

use crate::errors::BadInput;
use rc2k_core::strings::bytes::Bytes;
use rc2k_core::strings::case::CaseTable;
use std::cmp::Ordering;

/// The language's lower-case table (`g_lowerCase`, 0x4F0BC8). Until `strings::load_language` sets it up it is the image's:
/// every character itself, but A .. Z and 0xC0 .. 0xDE (not 0xD7) in lower case.
pub static LOWER: crate::session::Bytes<256> = crate::session::Bytes::new(first_lower());

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
    CaseTable {
        lower: std::array::from_fn(|c| LOWER.get(c)),
    }
}

/// A character in lower case (`g_lowerCase[c]`).
pub fn lower(c: u8) -> u8 {
    LOWER.get(c as usize)
}

/// `strings::load_language`'s new table.
pub fn set_case_table(table: &[u8; 256]) {
    for (c, &v) in table.iter().enumerate() {
        LOWER.set(c, v);
    }
}

/// How the first text's first differing character stands to the second's (converted:
/// `rc2k_core::strings::case::CaseTable::compare`).
pub fn compare_texts(
    a: &(impl Bytes + ?Sized),
    b: &(impl Bytes + ?Sized),
) -> Result<Ordering, BadInput> {
    fn read(text: &(impl Bytes + ?Sized)) -> Result<Vec<u8>, BadInput> {
        let len = crate::strings::text_length::nul_at(text)? as usize;
        let mut out = vec![0; len];
        text.copy_run(0, &mut out)
            .ok_or(BadInput("a text to compare past its block"))?;
        Ok(out)
    }
    Ok(case_table().compare(&read(a)?, &read(b)?))
}

/// `compare_texts` on two texts in hand (each up to its NUL or its end).
pub fn compare_bytes(a: &[u8], b: &[u8]) -> Ordering {
    let text = |t: &[u8]| {
        t.iter()
            .position(|&c| c == 0)
            .map_or(t, |n| &t[..n])
            .to_vec()
    };
    case_table().compare(&text(a), &text(b))
}

pub fn same_text(a: &(impl Bytes + ?Sized), b: &(impl Bytes + ?Sized)) -> Result<bool, BadInput> {
    Ok(compare_texts(a, b)? == Ordering::Equal)
}
