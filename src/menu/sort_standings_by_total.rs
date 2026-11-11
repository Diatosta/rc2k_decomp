//! `menu::sort_standings_by_total` (0x47B3D7): as `menu::sort_standings_by_time` by +0x18 (a reading: the total time).
//!
//! Refuses a null table, before anything.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;
pub fn sort_standings_by_total(menus: &Menus) -> Result<(), Refused> {
    sort_table_a(menus, rc2k_core::saves::entries::Field::RallyTime)?;
    Ok(())
}
