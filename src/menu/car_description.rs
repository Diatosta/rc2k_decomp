//! A car's description, 0x78 bytes: 34 of them, the last the Clio of the update disc (car
//! 0x21), pointed at by the car table and by the carousels' records (the carousel's car list).
//! The figures are in imperial units, in tenths; the strings are string ids. Read by the car
//! specification screens (`screens::draw_car_figures`, `car_caption`,
//! `menu::load_car_background`, `sound::play_car_stream`) and the carousels.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::lists;
use crate::menu::owner::Menus;

pub use rc2k_core::screens::car_pages::CarDescription;

const _: () = assert!(std::mem::size_of::<CarDescription>() == 0x78);

/// The carousels' entries: a table of 0x100 description handles (`cars_store::car_ref`).
const ENTRIES: u32 = 0x100;

/// The description of the entry shown (`EventState.shown.car` through `Carousel::entries`), by
/// handle.
pub fn shown_description(menus: &Menus) -> Result<u32, Refused> {
    let table = menus.screens.carousel.list()?;
    let entry: u32 = menus.screens.event.shown.car.get();
    let (Some(table), true) = (table, entry < ENTRIES) else {
        return Err(BadInput("no carousel records, or an entry past them").into());
    };
    let record = lists().car_entry(menus, table, entry)?;
    if record == 0 {
        return Err(BadInput("an entry without its record").into());
    }
    Ok(record)
}

/// A description by handle (`cars_store::car_ref`).
pub fn read(menus: &Menus, description: u32) -> Result<CarDescription, BadInput> {
    Ok(crate::menu::cars_store::description(&menus.cars.table, description)?.get())
}
