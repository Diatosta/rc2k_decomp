//! `menu::sort_standings_by_time`: standings table A's first 0xB4 entries sorted by their +0x14
//! (read as the stage time), as `menu::standings::sort_table_a`.
//!
//! Refuses a null table, before anything.

use crate::errors::Refused;
use crate::menu::owner::{Play, Screens};
use crate::menu::standings::*;
pub fn sort_standings_by_time(
    screens: &Screens,
    play: &Play,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    sort_table_a(screens, play, tables, rc2k_core::saves::entries::Field::Key)?;
    Ok(())
}
