//! `menu::clear_standings_b` (0x4A5A2F): each entry of standings table B (see `menu::standings`)
//! given +0x70 0.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers: the event drivers (`sub_4A0E60`, ..). Refuses a table without its end entry,
//! before anything. Self-test: `standings::test_clear` on a table of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_standings_b() -> Result<(), Refused> {
    let first: u32 = unsafe { crate::framework::game_read(TABLE_B_RVA) };
    let all = entries(first, B_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::B_FIGURE`, stored in each entry.
    use rc2k_core::menu::standings::B_FIGURE;
    unsafe { table(first, B_ENTRY, all.len() as u32) }.store_all(&[(B_FIGURE, 0)]);
    Ok(())
}
