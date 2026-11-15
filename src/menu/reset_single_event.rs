//! `menu::reset_single_event`: `champ_view.services` / `event.stages_done` 0; every
//! driver's +0x10 / +0xC / +0x20 0 and +0x24 1; `event.event_goes_on` 1, `event.event_out` 0; the
//! four players' `total` and `unread_84` 0 and `entry` 1 (`Player::begin`; a single event's
//! start).
//!
//! Refuses a missing player or a drivers' table without its end
//! entry, before anything.
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn reset_single_event(menus: &Menus) -> Result<(), Refused> {
    let drivers = drivers(menus)?;

    menus.play.champ_view.services.set(0u32);
    menus.screens.event.stages_done.set(0u32);

    drivers
        .iter()
        .for_each(crate::menu::drivers_store::Driver::begin);

    menus.screens.event.event_goes_on.set(1u32);
    menus.screens.event.event_out.set(0u32);

    for n in (0..4).rev() {
        player(menus, n).update(rc2k_core::menu::player::Player::begin);
    }
    Ok(())
}
