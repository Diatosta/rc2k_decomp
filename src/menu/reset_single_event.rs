//! `menu::reset_single_event` (0x4AAA2F): `dword_755254` / `dword_755950` 0; every
//! driver's +0x10 / +0xC / +0x20 0 and +0x24 1; `dword_75592C` 1, `dword_755978` 0; the
//! four players' +0x8C / +0x84 0 and +0xD0 1 (a reading: a single event's start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4AA4D7` (twice). Refuses a missing player or a drivers' table without its end
//! entry, before anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn reset_single_event(menus: &Menus) -> Result<(), Refused> {
    let drivers = drivers()?;

    menus.tables.pages.services.set(0u32);
    menus.screens.event.stages_done.set(0u32);

    drivers
        .iter()
        .for_each(crate::menu::drivers_store::Driver::begin);

    menus.screens.event.event_goes_on.set(1u32);
    menus.screens.event.event_out.set(0u32);

    for n in (0..4).rev() {
        player(n).update(rc2k_core::menu::player::Player::begin);
    }
    Ok(())
}
