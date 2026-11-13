//! Appends an unsigned decimal number followed by '%'.

pub fn append_percent(value: u32) -> Vec<u8> {
    rc2k_core::strings::format::percent(value)
}
