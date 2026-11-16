//! `menu::sort_standings_b_by_points`: standings table B's first 0xB4 entries sorted going down by
//! their +0x6C (the points' total), as `menu::standings::sort_table_b`.
//!
//! Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;
pub fn sort_standings_b_by_points(
    menus: &Menus,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    sort_table_b(
        &menus.screens,
        &menus.play,
        tables,
        rc2k_core::saves::rankings::BKey::StageTotal,
        true,
    )?;
    Ok(())
}
