//! `menu::reset_drivers` (0x470399): every driver (`g_drivers`) back in the event (+0x24
//! 1) with no stage or total time (+0xC, +0x10 0): `Driver::reset`.
//!
//! Callers: the event drivers. Refuses a table without its end entry, before
//! anything.

use crate::errors::Refused;
use crate::menu::drivers_store::Driver;
use crate::menu::owner::Menus;
use crate::menu::standings::drivers;

pub fn reset_drivers(menus: &Menus) -> Result<(), Refused> {
    drivers(menus)?.iter().for_each(Driver::reset);
    Ok(())
}
