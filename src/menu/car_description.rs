//! A car's description, 0x78 bytes (`CarDescription` in IDA): 34 of them from 0x756568,
//! the last the Clio of the update disc (car 0x21), pointed at by the table at
//! 0x7564A0 and by the carousels' records (`dword_75A65C`). The figures are in imperial
//! units, in tenths; the strings are string ids. Read by the car specification screens
//! (`screens::draw_car_figures`, `car_caption`, `menu::load_car_background`,
//! `sound::play_car_stream`) and the carousels.

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::state_store::MENU_STATE;

pub use rc2k_core::screens::car_pages::CarDescription;

const _: () = assert!(std::mem::size_of::<CarDescription>() == 0x78);

/// The carousels' entries: a table of 0x100 description handles (`cars_store::car_ref`).
const ENTRIES: u32 = 0x100;

/// The description of the entry shown (`dword_755990` through `dword_75A65C`), by handle.
pub fn shown_description() -> Result<u32, Refused> {
    let table = MENU_STATE.carousel.list()?;
    let entry: u32 = EVENT_STATE.shown_car.get();
    let (Some(table), true) = (table, entry < ENTRIES) else {
        return Err(BadInput("no carousel records, or an entry past them").into());
    };
    let record = crate::menu::lists_store::MENU_LISTS
        .cars(table)
        .offset(entry * 4)
        .read::<u32>("a car description handle")?;
    if record == 0 {
        return Err(BadInput("an entry without its record").into());
    }
    Ok(record)
}

/// A description by handle (`cars_store::car_ref`).
pub fn read(description: u32) -> Result<CarDescription, BadInput> {
    Ok(crate::menu::cars_store::description(description)?.get())
}
