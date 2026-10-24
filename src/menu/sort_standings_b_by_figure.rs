//! `menu::sort_standings_b_by_figure` (0x4A467A): standings table B's first 0xB4 entries sorted going up by their +0x34 (a reading: the stage's figure), as
//! `menu::standings::sort_table_b`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: nine of the game's. Refuses a null table, before anything. Self-test: the original and
//! the Rust version on the game's tables, keys by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::menu::standings::*;

// dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn sort_standings_b_by_figure() -> Result<(), Refused> {
    sort_table_b(0x34, false)?;
    Ok(())
}
