//! `menu::reset_driver_condition` (0x47E7BC): every driver's (`g_drivers`) twelve figures
//! +0x2C .. +0x58 set to 100 (a reading: the cars' condition at an event's start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_471F0D` (twice), the event drivers. Refuses a table without its end entry, before
//! anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn reset_driver_condition() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::event::driver_condition`, stored in each driver.
    driver_table()?.store_all(&rc2k_core::menu::event::driver_condition());
    Ok(())
}
