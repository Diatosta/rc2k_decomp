//! `menu::sort_standings_b_by_figure`: standings table B's first 0xB4 entries sorted going up by
//! their +0x34 (a reading: the stage's figure), as `menu::standings::sort_table_b`.
//!
//! Callers: nine of the game's. Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;
pub fn sort_standings_b_by_figure(
    menus: &Menus,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    sort_table_b(
        &menus.screens,
        &menus.play,
        tables,
        rc2k_core::saves::rankings::BKey::Total,
        false,
    )?;
    Ok(())
}
