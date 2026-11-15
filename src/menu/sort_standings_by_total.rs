//! `menu::sort_standings_by_total`: as `menu::sort_standings_by_time` by +0x18 (a reading: the
//! total time).
//!
//! Refuses a null table, before anything.

use crate::errors::Refused;
use crate::menu::owner::{Play, Screens};
use crate::menu::standings::*;
pub fn sort_standings_by_total(screens: &Screens, play: &Play) -> Result<(), Refused> {
    sort_table_a(screens, play, rc2k_core::saves::entries::Field::RallyTime)?;
    Ok(())
}
