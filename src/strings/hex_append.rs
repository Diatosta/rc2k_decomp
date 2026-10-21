//! `strings::hex_append` (0x40211D): appends `digits` uppercase hex digits of a value, MSB first.
//!
//! In:  eax = value, ecx = digits (0..=8), ebx = cursor
//! Out: ebx = new cursor
//! More than 8 digits is refused.
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

use crate::errors::BadInput;

// ---- Rust version ----

/// The low `digits` hex digits of `value`, most significant first.
pub fn hex_append(value: u32, digits: u32) -> Result<Vec<u8>, BadInput> {
    rc2k_core::strings::format::hex(value, digits).ok_or(BadInput(
        "more than 8 hex digits: not worked out in the original",
    ))
}
