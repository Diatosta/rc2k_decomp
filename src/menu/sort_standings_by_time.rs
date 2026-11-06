//! `menu::sort_standings_by_time` (0x47B2C4): standings table A's first 0xB4 entries sorted by their +0x14 (a reading: the stage time), as `menu::standings::sort_table_a`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_473126`, `sub_4A1EE6`. Refuses a null table, before anything.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

// dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub fn sort_standings_by_time(menus: &Menus) -> Result<(), Refused> {
    sort_table_a(menus, rc2k_core::saves::entries::Field::Key)?;
    Ok(())
}
