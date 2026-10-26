//! `marks::pong_digit` (0x4B8418): one score digit: `marks::pong_cell` for its 4 x 5
//! pattern (`unk_768704`, 0x14 bytes a digit; ebx the column, ecx the row, eax walking
//! the pattern), then `dword_7687D4` on 0x40.
//!
//! In:  eax the digit
//! Out: nothing (all kept: `pusha` .. `popa`)
//!
//! Caller: `marks::pong_score` (six times). Refuses a digit past 9 (the patterns' end),
//! before anything.

use crate::errors::Refused;
use crate::marks::pong::PONG;

/// The Rust version (see the header): `digit` (eax).
// Converted: `rc2k_core::marks::pong::{pattern, cells}`
pub fn pong_digit(digit: u32) -> Result<(), Refused> {
    use rc2k_core::marks::pong as core;
    let mut pattern = PONG.digit_patterns.addr() + core::pattern(digit)?;
    for (col, row) in core::cells() {
        crate::marks::pong_cell::pong_cell(pattern, col, row)?;
        pattern = pattern.wrapping_add(1);
    }
    PONG.digit_at[0].set(PONG.digit_at[0].get().wrapping_add(core::DIGIT_STEP));
    Ok(())
}
