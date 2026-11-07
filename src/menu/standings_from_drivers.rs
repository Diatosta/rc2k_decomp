//! `menu::standings_from_drivers` (0x4A9605): standings table A emptied (`menu::init_standings_a`) and filled from the drivers in order: +0x14 / +0x1C / +0x18 / +8 / +0x10 / +0 / +0x4C from the driver's +0xC / +8 / +0x10 / +0x14 / +0x18 / +0 / +0x24, +4 0, +0xC the driver's name string (`g_driverNames`, 0x14 each, by its +0) and +0x20 its text.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. `strings::append_string_by_id`'s Rust version. Refuses no table, more
//! drivers than its 200 entries, a name past the entry, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn standings_from_drivers(menus: &Menus) -> Result<(), Refused> {
    let drivers = walked_drivers()?;
    if drivers.len() > 200 {
        return Err(BadInput("more drivers than the standings table holds").into());
    }
    let mut texts = Vec::new();
    for d in &drivers {
        let id: u32 = crate::menu::drivers_store::name_string(menus, d.number)?;
        let mut text = Vec::new();
        crate::strings::append_string_by_id::push_string(&mut text, id)?;
        text.push(0);
        if text.len() > rc2k_core::saves::entries::TIME_LEN - 0x20 {
            return Err(BadInput("a driver's name past its entry").into());
        }
        texts.push((id, text));
    }
    crate::menu::init_standings_a::init_standings_a()?;
    // Converted: `rc2k_core::saves::rankings::fill_a_from_driver`.
    with_a(|t| {
        for ((d, (id, text)), e) in drivers.iter().zip(&texts).zip(&mut t.entries) {
            rc2k_core::saves::rankings::fill_a_from_driver(e, d, *id, text)
                .expect("the name's length checked");
        }
    })?;
    Ok(())
}
