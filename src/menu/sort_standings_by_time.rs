//! `menu::sort_standings_by_time` (0x47B2C4): standings table A's first 0xB4 entries sorted by their +0x14 (a reading: the stage time), as `menu::standings::sort_table_a`.
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
use crate::menu::standings::*;

// dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn sort_standings_by_time() -> Result<(), Refused> {
    sort_table_a(0x14)?;
    Ok(())
}
