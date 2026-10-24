//! `menu::sort_standings_b_by_score` (0x4A5C77): standings table B's first 0xB4 entries sorted going down by their +0x50, as
//! `menu::standings::sort_table_b`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: two of the game's, `sub_4A9583`. Refuses a null table, before anything. Self-test: the original and
//! the Rust version on the game's tables, keys by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::menu::standings::*;

// dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn sort_standings_b_by_score() -> Result<(), Refused> {
    sort_table_b(0x50, true)?;
    Ok(())
}
