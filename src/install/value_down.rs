//! `install::value_down` (0x414881): an item of a value (kind 3) lowered by the key's
//! repeats (`g_listRepeats`), not below its least (`[esi+0Ch]`, signed compare).
//!
//! In:  esi the item (+8 the value's byte, +0xC the least)
//! Out: edx the value's address, ebx the least, ecx the new value; zero set, carry clear
//! Callers `install::key_left`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::installer::{REPEATS, g, rb, rd, wb};

pub fn value_down(item: u32) {
    let value_at = rd(item + 8);
    // Converted: `rc2k_core::install::items::value_down`.
    let value = rc2k_core::install::items::value_down(rb(value_at), g(REPEATS), rb(item + 0xC));
    wb(value_at, value as u8);
}
