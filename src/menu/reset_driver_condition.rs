//! `menu::reset_driver_condition`: every driver's twelve `condition` figures
//! set to 100 (a reading: the cars' condition at an event's start).
//!
//! Refuses a table without its end entry, before anything.
use crate::errors::Refused;
use crate::menu::drivers_store::Driver;
use crate::menu::owner::Menus;
use crate::menu::standings::walked;

pub(super) fn reset_driver_condition(menus: &Menus) -> Result<(), Refused> {
    let count = walked(&menus.cars)?;
    menus.cars.drivers.each(count, Driver::reset_condition);
    Ok(())
}
