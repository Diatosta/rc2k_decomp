//! Appends an unsigned decimal number. Flags: low nibble = width, 0x10 = zero-pad to width, 0x20 =
//! space-pad to width, neither = no padding. 0x40 and 0x80 are ignored.

pub fn format_u32(value: u32, flags: u8) -> Vec<u8> {
    rc2k_core::strings::format::decimal(value, rc2k_core::strings::format::Pad::from_flags(flags))
}
