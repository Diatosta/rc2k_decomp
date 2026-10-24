//! `install::value_up` (0x41489C): an item of a value (kind 3) raised by the key's repeats
//! (`g_listRepeats`), not above its most (`[esi+0Dh]`, signed compare).
//!
//! In:  esi the item (+8 the value's byte, +0xD the most)
//! Out: edx the value's address, ebx the most, ecx the new value; zero set, carry clear
//! Callers `install::key_right`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{REPEATS, g, rb, rd, wb};

pub fn value_up(item: u32) {
    let value_at = rd(item + 8);
    // Converted: `rc2k_core::install::items::value_up`.
    let value = rc2k_core::install::items::value_up(rb(value_at), g(REPEATS), rb(item + 0xD));
    wb(value_at, value as u8);
}
