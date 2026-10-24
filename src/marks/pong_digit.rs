//! `marks::pong_digit` (0x4B8418): one score digit: `marks::pong_cell` for its 4 x 5
//! pattern (`unk_768704`, 0x14 bytes a digit; ebx the column, ecx the row, eax walking
//! the pattern), then `dword_7687D4` on 0x40.
//!
//! In:  eax the digit
//! Out: nothing (all kept: `pusha` .. `popa`)
//!
//! Caller: `marks::pong_score` (six times). The cells through `part` (1), in the game
//! their Rust version. Refuses a digit past 9 (the patterns' end), before anything.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose cell calls go
//! to a stand-in, against the Rust version, every digit.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::marks::pong;
use crate::marks::store::{game_addr, game_read, game_write};

/// The Rust version (see the header): `digit` (eax).
// Converted: `rc2k_core::marks::pong::{pattern, cells}`
pub unsafe fn pong_digit(digit: u32) -> Result<(), Refused> {
    use rc2k_core::marks::pong as core;
    let mut pattern = game_addr(pong::PATTERNS_RVA) as u32 + core::pattern(digit)?;
    for (col, row) in core::cells() {
        unsafe { crate::marks::pong_cell::pong_cell(pattern, col, row)? };
        pattern = pattern.wrapping_add(1);
    }
    unsafe {
        let at: u32 = game_read(pong::DIGIT_AT[0]);
        game_write(pong::DIGIT_AT[0], at.wrapping_add(core::DIGIT_STEP));
    }
    Ok(())
}
