//! Two dwords as "AABB-CCDD-EEFF-GGHH" in the seal text buffer: each dword's bytes from the
//! lowest, two hex digits each, a '-' after every second byte; no NUL. The stage results page shows a seal code with it.

use crate::strings::owner::Strings;

/// The seal text buffer's room (`Text::seal_text`); the byte after the 19 characters is the text's
/// NUL.
pub(super) const SEAL_TEXT_LEN: usize = 0x14;

/// The 19 characters.
pub fn seal_code(a: u32, b: u32) -> Vec<u8> {
    rc2k_core::strings::format::seal_code(a, b)
}

/// The two dwords as the seal text in the strings' buffer. (The race keeps the dwords in its
/// seal cipher block: the caller sets them.)
pub fn format_seal_code(strings: &Strings, a: u32, b: u32) -> Result<(), crate::errors::BadInput> {
    let code = seal_code(a, b);
    strings.text.seal_text.write_at(0, &code, "the seal text")
}
