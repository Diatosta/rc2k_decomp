//! `strings::append_percent` (0x401ACB): appends an unsigned decimal number followed by '%'.
//!
//! In:  eax = value, ebx = cursor
//! Out: ebx = new cursor; everything else preserved
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

// ---- Rust version ----

pub fn append_percent(value: u32) -> Vec<u8> {
    rc2k_core::strings::format::percent(value)
}
