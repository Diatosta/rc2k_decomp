//! `hud::tenths` (0x4B1D2B): six times of the HUD's record in tenths (signed `idiv` by
//! 10): +0x24, +0x2C, +0x20, +0x54, +0x58, +0x50 into `dword_75C074` .. `dword_75C088`.
//!
//! In:  ebp the HUD's record
//! Out: eax the last quotient, edx its remainder, ebx 10; the rest kept
//!
//! Caller: `hud::draw_hud`. Self-test: the original and the Rust version on records of the
//! test's, negative times among them.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_write, rva};

const OUT_RVA: usize = rva(0x75C074); // dword_75C074 .. dword_75C088
const FIELDS: [u32; 6] = [0x24, 0x2C, 0x20, 0x54, 0x58, 0x50];

/// The Rust version (see the header): the record's six times in tenths.
pub unsafe fn tenths(record: u32) {
    // Converted: `rc2k_core::hud::tenths::tenths`.
    let times = FIELDS.map(|f| unsafe { ((record + f) as *const i32).read_unaligned() });
    let (tenths, _) = rc2k_core::hud::tenths::tenths(times);
    unsafe { game_write(OUT_RVA, tenths) };
}
