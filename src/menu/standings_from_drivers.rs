//! `menu::standings_from_drivers` (0x4A9605): standings table A emptied (`menu::init_standings_a`) and filled from the drivers in order: +0x14 / +0x1C / +0x18 / +8 / +0x10 / +0 / +0x4C from the driver's +0xC / +8 / +0x10 / +0x14 / +0x18 / +0 / +0x24, +4 0, +0xC the driver's name string (`g_driverNames`, 0x14 each, by its +0) and +0x20 its text.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. `strings::append_string_by_id`'s Rust version. Refuses no table, more
//! drivers than its 200 entries, a name past the entry, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn standings_from_drivers() -> Result<(), Refused> {
    let first: u32 = { TABLE_A_RVA.get() };
    let drivers = entries(crate::menu::drivers_store::DRIVERS.first(), DRIVER)?;
    if first == 0 || drivers.len() > 200 {
        return Err(BadInput("no standings table, or more drivers than it holds").into());
    }
    let mut texts = Vec::new();
    for &d in &drivers {
        let id: u32 = crate::menu::drivers_store::name_string(get(d));
        let mut text = Vec::new();
        unsafe { crate::strings::append_string_by_id::push_string(&mut text, id) };
        text.push(0);
        if text.len() > (A_ENTRY - 0x20) as usize {
            return Err(BadInput("a driver's name past its entry").into());
        }
        texts.push((id, text));
    }
    unsafe { crate::menu::init_standings_a::init_standings_a() }?;
    // Converted: `rc2k_core::menu::standings::fill_a_from_driver`.
    let count = drivers.len() as u32;
    let all = unsafe { table(drivers[0], DRIVER, count) };
    let mut t = unsafe { table(first, A_ENTRY, count) };
    for (n, (id, text)) in texts.into_iter().enumerate() {
        rc2k_core::menu::standings::fill_a_from_driver(&mut t, n, &all, n, id, &text);
    }
    Ok(())
}
