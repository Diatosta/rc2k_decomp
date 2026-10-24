//! `menu::standings_from_drivers` (0x4A9605): standings table A emptied (`menu::init_standings_a`) and filled from the drivers in order: +0x14 / +0x1C / +0x18 / +8 / +0x10 / +0 / +0x4C from the driver's +0xC / +8 / +0x10 / +0x14 / +0x18 / +0 / +0x24, +4 0, +0xC the driver's name string (`g_driverNames`, 0x14 each, by its +0) and +0x20 its text.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. `strings::append_string_by_id`'s Rust version. Refuses no table, more
//! drivers than its 200 entries, a name past the entry, before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::menu::standings::*;

const NAMES_RVA: usize = rva(0x741E88); // g_driverNames

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn standings_from_drivers() -> Result<(), Refused> {
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    let drivers = entries(game_addr(DRIVERS_RVA) as u32, DRIVER)?;
    if first == 0 || drivers.len() > 200 {
        return Err(BadInput("no standings table, or more drivers than it holds").into());
    }
    let mut texts = Vec::new();
    for &d in &drivers {
        let id: u32 = unsafe { game_read(NAMES_RVA + 0x14 * get(d) as usize) };
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
