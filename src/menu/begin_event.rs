//! `menu::begin_event` (0x472BAA): the drivers and the four players made ready for a new
//! event.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755254` and `dword_755950` 0; every driver of `g_drivers` (`standings::drivers`'s walk)
//! gets +0x10, +0xC, +0x20 = 0 and +0x24 = 1 (`Driver::begin`); `dword_75592C` 1,
//! `dword_755978` 0; then for each player n (the current player `dword_75598C` = n, left at 3):
//! `field_8c` and `field_84` 0, +0xD0 1, `service::repair_car`, and unless the event mode
//! `dword_7556D0` is 3 the setup's defaults: gear ratio 2, brake balance 0x32, tyre 3, ride height
//! 0x46, suspension 0x1E, steering sensitivity = the controller's (+0x280). Callers: the event
//! loops' starts (`sub_482382` and nine more). Refuses drivers without their end entry, a
//! missing player.

use crate::errors::Refused;
use crate::menu::player::PLAYERS;
use crate::session::Dword;
use rc2k_core::menu::event::KEEPS_SETUP;
use rc2k_core::menu::player::Player;

const CURRENT_PLAYER: &Dword = crate::menu::take_shown_car::CURRENT_PLAYER_RVA;
pub fn begin_event() -> Result<(), Refused> {
    let drivers = crate::menu::standings::drivers()?;
    crate::menu::owner::menus().tables.pages.services.set(0u32);
    crate::menu::owner::menus()
        .screens
        .event
        .stages_done
        .set(0u32);
    // Converted: `rc2k_core::menu::player::Player::{begin, default_setup}`.
    drivers
        .iter()
        .for_each(crate::menu::drivers_store::Driver::begin);
    crate::menu::owner::menus()
        .screens
        .event
        .event_goes_on
        .set(1u32);
    crate::menu::owner::menus()
        .screens
        .event
        .event_out
        .set(0u32);
    for n in 0..PLAYERS {
        CURRENT_PLAYER.set(n);
        let p = crate::menu::standings::player(n);
        p.update(Player::begin);
        crate::service::repair_parts::repair_car()?;
        if crate::menu::owner::menus().play.champ_view.event_mode.get() != KEEPS_SETUP {
            p.update(Player::default_setup);
        }
    }
    Ok(())
}
