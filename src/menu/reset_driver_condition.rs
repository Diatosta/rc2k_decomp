//! `menu::reset_driver_condition` (0x47E7BC): every driver's (`g_drivers`) twelve figures
//! +0x2C .. +0x58 set to 100 (a reading: the cars' condition at an event's start).
//!
//! Callers: the event drivers. Refuses a table without its end entry, before
//! anything.
use crate::errors::Refused;
use crate::menu::drivers_store::Driver;
use crate::menu::standings::drivers;

pub fn reset_driver_condition() -> Result<(), Refused> {
    drivers()?.iter().for_each(Driver::reset_condition);
    Ok(())
}
