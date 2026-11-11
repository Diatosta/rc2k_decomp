//! `menu::begin_event` (0x472BAA): the drivers and the four players made ready for a new
//! event.
//!
//! `pages.services` and `event.stages_done` 0; every driver of `g_drivers` (`standings::drivers`'s walk)
//! gets +0x10, +0xC, +0x20 = 0 and +0x24 = 1 (`Driver::begin`); `event.event_goes_on` 1,
//! `event.event_out` 0; then for each player n (the current player `event.current_player` = n, left at 3):
//! `total` and `unread_84` 0, `entry` 1 (`Player::begin`), `service::repair_car`, and unless the event mode
//! the event mode is 3 the setup's defaults: gear ratio 2, brake balance 0x32, tyre 3, ride height
//! 0x46, suspension 0x1E, steering sensitivity = the controller's (+0x280). Callers: the event
//! loops' starts (`menu::single_rally` and nine more). Refuses drivers without their end entry, a
//! missing player.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player::PLAYERS;
use crate::session::PlainDword;
use rc2k_core::menu::event::KEEPS_SETUP;
use rc2k_core::menu::player::Player;

const CURRENT_PLAYER: &PlainDword = crate::menu::take_shown_car::CURRENT_PLAYER_RVA;
pub fn begin_event(menus: &Menus) -> Result<(), Refused> {
    let drivers = crate::menu::standings::drivers(menus)?;
    menus.tables.pages.services.set(0u32);
    menus.screens.event.stages_done.set(0u32);
    // The logic is in `rc2k_core::menu::player::Player::{begin, default_setup}`.
    drivers
        .iter()
        .for_each(crate::menu::drivers_store::Driver::begin);
    menus.screens.event.event_goes_on.set(1u32);
    menus.screens.event.event_out.set(0u32);
    for n in 0..PLAYERS {
        CURRENT_PLAYER.set(n);
        let p = crate::menu::standings::player(menus, n);
        p.update(Player::begin);
        crate::service::repair_parts::repair_car()?;
        if menus.play.champ_view.event_mode.get() != KEEPS_SETUP {
            p.update(Player::default_setup);
        }
    }
    Ok(())
}
