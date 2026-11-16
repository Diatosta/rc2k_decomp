//! `menu::standings_from_drivers`: standings table A emptied (`menu::init_standings_a`) and filled
//! from the drivers in order: +0x14 / +0x1C / +0x18 / +8 / +0x10 / +0 / +0x4C from the driver's
//! +0xC / +8 / +0x10 / +0x14 / +0x18 / +0 / +0x24, +4 0, +0xC the driver's name string (the driver
//! names, 0x14 each, by its +0) and +0x20 its text.
//!
//! Refuses no table, more drivers than its 200 entries, a name past the entry, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Cars;
use crate::menu::standings::*;

pub fn standings_from_drivers(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    cars: &Cars,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let drivers = walked_drivers(cars)?;
    if drivers.len() > 200 {
        return Err(BadInput("more drivers than the standings table holds").into());
    }
    let mut texts = Vec::new();
    for d in &drivers {
        let id: u32 = crate::menu::drivers_store::name_string(cars, d.number)?;
        let mut text = Vec::new();
        crate::strings::append_string_by_id::push_string(so, &mut text, id)?;
        text.push(0);
        if text.len() > rc2k_core::saves::entries::TIME_LEN - 0x20 {
            return Err(BadInput("a driver's name past its entry").into());
        }
        texts.push((id, text));
    }
    crate::menu::init_standings_a::init_standings_a(tables)?;
    with_a(tables, |t| {
        for ((d, (id, text)), e) in drivers.iter().zip(&texts).zip(&mut t.entries) {
            rc2k_core::saves::rankings::fill_a_from_driver(e, d, *id, text)
                .expect("the name's length checked");
        }
    })?;
    Ok(())
}
