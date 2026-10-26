//! `menu::reset_drivers` (0x470399): every driver (`g_drivers`, 0x94 each) given +0x24 1,
//! +0xC 0 and +0x10 0 (a reading: in the event, no points, no place).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers and `sub_4A9C9B`. Refuses a table without its end entry, before
//! anything.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn reset_drivers() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::event::DRIVER_RESET`, stored in each driver.
    driver_table()?.store_all(&rc2k_core::menu::event::DRIVER_RESET);
    Ok(())
}
