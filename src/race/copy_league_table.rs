//! `race::copy_league_table` (0x4650B7): from the records at `dword_75ACD0` (ten of
//! 0x50; none when 0) each one's +8 into `dword_60410C` and its sixteen dwords from
//! +0x10 into its column of the sixteen rows at 0x604134 (0x28 apart).
//!
//! In:  nothing
//! Out: esi = past the records (0 without), ecx = 10, edx = 16, edi = past the last
//!      column, eax = the last dword; the rest preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_446EBE` (`pusha` follows). Shadow mode:
//! the Rust version, the tables put back, the original's, compared. Self-test: records of
//! the test's and none, the game's tables put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { copy_league_table(r) };
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
