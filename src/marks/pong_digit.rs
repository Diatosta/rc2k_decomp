//! `marks::pong_digit` (0x4B8418): one score digit: `marks::pong_cell` for its 4 x 5
//! pattern (`unk_768704`, 0x14 bytes a digit; ebx the column, ecx the row, eax walking
//! the pattern), then `dword_7687D4` on 0x40.
//!
//! In:  eax the digit
//! Out: all kept (`pusha` .. `popa`)
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
use crate::framework::{Regs, game_addr, game_read, game_write};
use crate::marks::pong;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::pong::{pattern, cells}`
pub unsafe fn pong_digit(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::marks::pong as core;
    let mut r = *entry;
    r.eax = game_addr(pong::PATTERNS_RVA) as u32 + core::pattern(entry.eax)?;
    for (col, row) in core::cells() {
        r.ebx = col;
        r.ecx = row;
        crate::marks::pong_digit::game_part(1, &mut r)?;
    }
    unsafe {
        let at: u32 = game_read(pong::DIGIT_AT[0]);
        game_write(pong::DIGIT_AT[0], at.wrapping_add(core::DIGIT_STEP));
    }
    Ok(*entry)
}

/// In the game: `pong_cell`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::pong_cell as pc;
    *r = unsafe { pc::pong_cell(r) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { pong_digit(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
