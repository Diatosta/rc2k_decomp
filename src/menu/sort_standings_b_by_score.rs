//! `menu::sort_standings_b_by_score`: standings table B's first 0xB4 entries sorted going down by
//! their +0x50, as `menu::standings::sort_table_b`.
//!
//! Callers: two of the game's. Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;
pub fn sort_standings_b_by_score(menus: &Menus) -> Result<(), Refused> {
    sort_table_b(menus, rc2k_core::saves::rankings::BKey::OverallTotal, true)?;
    Ok(())
}
