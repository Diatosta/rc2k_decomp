//! `menu::reset_drivers`: every driver back in the event (+0x24 set to 1) with no stage or
//! total time (+0xC, +0x10 set to 0): `Driver::reset`.
//!
//! Refuses a table without its end entry, before anything.

use crate::errors::Refused;
use crate::menu::drivers_store::Driver;
use crate::menu::owner::Menus;
use crate::menu::standings::drivers;

pub(super) fn reset_drivers(menus: &Menus) -> Result<(), Refused> {
    drivers(menus)?.iter().for_each(Driver::reset);
    Ok(())
}
