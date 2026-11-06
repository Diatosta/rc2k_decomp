//! `effects::pong_digit` (0x4B8418): one score digit: `effects::pong_cell` for its 4 x 5
//! pattern (`unk_768704`, 0x14 bytes a digit; ebx the column, ecx the row, eax walking
//! the pattern), then `dword_7687D4` on 0x40.
//!
//! In:  eax the digit
//! Out: nothing (all kept: `pusha` .. `popa`)
//!
//! Caller: `effects::pong_score` (six times). Refuses a digit past 9 (the patterns' end),
//! before anything.

use crate::effects::pong::PONG;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;

/// The Rust version (see the header): `digit` (eax).
// Converted: `rc2k_core::effects::pong::{pattern, cells}`
pub fn pong_digit(fl: &WindscreenFlakes, digit: u32) -> Result<(), Refused> {
    use rc2k_core::effects::pong as core;
    let first = core::pattern(digit)? as usize;
    for (n, (col, row)) in core::cells().enumerate() {
        crate::effects::pong_cell::pong_cell(fl, PONG.digit_patterns.get(first + n), col, row)?;
    }
    PONG.digit_at[0].set(PONG.digit_at[0].get().wrapping_add(core::DIGIT_STEP));
    Ok(())
}
