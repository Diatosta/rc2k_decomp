//! `menu::reset_drivers` (0x470399): every driver (`g_drivers`) back in the event (+0x24
//! 1) with no stage or total time (+0xC, +0x10 0): `Driver::reset`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers and `sub_4A9C9B`. Refuses a table without its end entry, before
//! anything.

use crate::errors::Refused;
use crate::menu::drivers_store::Driver;
use crate::menu::standings::drivers;

/// The Rust version (see the header): the registers it leaves.
pub fn reset_drivers() -> Result<(), Refused> {
    drivers()?.iter().for_each(Driver::reset);
    Ok(())
}
