//! `menu::sort_standings_b_by_figure` (0x4A467A): standings table B's first 0xB4 entries sorted going up by their +0x34 (a reading: the stage's figure), as
//! `menu::standings::sort_table_b`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: nine of the game's. Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

// dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub fn sort_standings_b_by_figure() -> Result<(), Refused> {
    sort_table_b(rc2k_core::saves::rankings::BKey::Total, false)?;
    Ok(())
}
