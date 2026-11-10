//! `strings::append_percent` (0x401ACB): appends an unsigned decimal number followed by '%'.
//!
//! Returns new cursor.

pub fn append_percent(value: u32) -> Vec<u8> {
    rc2k_core::strings::format::percent(value)
}
