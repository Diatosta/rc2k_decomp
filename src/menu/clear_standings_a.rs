//! `menu::clear_standings_a` (0x4A5A14): each entry of standings table A (see `menu::standings`)
//! given +0x44 0.
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
use crate::framework::Regs;
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_standings_a(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { crate::framework::game_read(TABLE_A_RVA) };
    let all = entries(first, A_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::A_FIGURE`, stored in each entry.
    use rc2k_core::menu::standings::A_FIGURE;
    unsafe { table(first, A_ENTRY, all.len() as u32) }.store_all(&[(A_FIGURE, 0)]);
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_standings_a(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
