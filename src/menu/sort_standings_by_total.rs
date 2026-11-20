//! `menu::sort_standings_by_total`: as `menu::sort_standings_by_time` by +0x18 (a reading: the
//! total time).
//!
//! Refuses a null table, before anything.

use crate::errors::Refused;
use crate::menu::owner::{Play, Screens};
use crate::menu::standings::*;
pub(super) fn sort_standings_by_total(
    screens: &Screens,
    play: &Play,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    sort_table_a(
        screens,
        play,
        tables,
        rc2k_core::saves::entries::Field::RallyTime,
    )?;
    Ok(())
}
