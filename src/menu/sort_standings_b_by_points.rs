//! `menu::sort_standings_b_by_points` (0x4A5B07): standings table B's first 0xB4 entries sorted going down by their +0x6C (the points' total), as
//! `menu::standings::sort_table_b`.
//!
//! Callers: three of the game's. Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

// dword_75598C

/// The Rust version.
pub fn sort_standings_b_by_points(menus: &Menus) -> Result<(), Refused> {
    sort_table_b(menus, rc2k_core::saves::rankings::BKey::StageTotal, true)?;
    Ok(())
}
