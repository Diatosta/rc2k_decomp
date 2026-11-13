//! A text coded by rot13 (letters 13 on, `A` .. `Z` and `a` .. `z` each in their own round; other
//! bytes as they are) into the shared text buffer (`text`), with its NUL.

use crate::errors::BadInput;
use crate::strings::text_from_ids::TEXT_LEN;
use rc2k_core::strings::bytes::Bytes;

/// One byte coded.
pub fn rot13(b: u8) -> u8 {
    rc2k_core::strings::format::rot13(b)
}

/// The coded text with its NUL, also left in the text buffer.
pub fn rot13_into_text(
    strings: &crate::strings::owner::Strings,
    source: &impl Bytes,
) -> Result<Vec<u8>, BadInput> {
    let mut coded = Vec::new();
    loop {
        let b = source
            .byte(coded.len())
            .ok_or(BadInput("a text without its end in its block"))?;
        coded.push(rot13(b));
        if b == 0 {
            break;
        }
        if coded.len() >= TEXT_LEN {
            return Err(BadInput("a text longer than the text buffer"));
        }
    }
    for (i, &b) in coded.iter().enumerate() {
        strings.text.buffer.text.set(i, b);
    }
    Ok(coded)
}
