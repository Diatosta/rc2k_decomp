//! `strings::format_u32` (0x40204A): appends an unsigned decimal number.
//!
//! In:  eax = value, ecx = flags, ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! Flags: low nibble = width, 0x10 = zero-pad to width, 0x20 = space-pad to width,
//! neither = no padding. 0x40 and 0x80 are ignored.

// ---- Rust version ----

/// Converted: `rc2k_core::strings::format::decimal` (the flags byte as `Pad`).
pub fn format_u32(value: u32, flags: u8) -> Vec<u8> {
    rc2k_core::strings::format::decimal(value, rc2k_core::strings::format::Pad::from_flags(flags))
}
