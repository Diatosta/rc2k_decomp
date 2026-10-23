//! `menu::sort_standings_by_total` (0x47B3D7): as `menu::sort_standings_by_time` by +0x18 (a reading: the total time).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_473126`, `sub_4A1EE6`. Refuses a null table, before anything. Self-test: the original and the Rust version on the game's tables, keys by lot; put
//! back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::*;

// dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn sort_standings_by_total(entry: &Regs) -> Result<Regs, Refused> {
    sort_table_a(0x18)?;
    Ok(*entry)
}
