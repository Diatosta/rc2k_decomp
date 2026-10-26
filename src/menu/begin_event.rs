//! `menu::begin_event` (0x472BAA): the drivers and the four players made ready for a new
//! event.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755254` and `dword_755950` 0; every driver of `g_drivers` (0x94 bytes each, up to the one
//! whose +0 is -1; the first always) gets +0x10, +0xC, +0x20 = 0 and +0x24 = 1; `dword_75592C` 1,
//! `dword_755978` 0; then for each player n (the current player `dword_75598C` = n, left at 3):
//! `field_8c` and `field_84` 0, +0xD0 1, `service::repair_car`, and unless the event mode
//! `dword_7556D0` is 3 the setup's defaults: gear ratio 2, brake balance 0x32, tyre 3, ride height
//! 0x46, suspension 0x1E, steering sensitivity = the controller's (+0x280). Callers: the event
//! loops' starts (`sub_482382` and nine more). Refuses drivers without their end within 0x400, a
//! missing player.

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::player::{PLAYERS, player_at};
use crate::menu::standings::put_all;
use crate::session::Dword;
use rc2k_core::menu::event::KEEPS_SETUP;

pub const DRIVER_LEN: u32 = 0x94;
const MOST_DRIVERS: u32 = 0x400;
const CURRENT_PLAYER: &Dword = crate::menu::take_shown_car::CURRENT_PLAYER_RVA;
/// The drivers' count (with the first), or `None` without their end.
pub fn drivers() -> Result<Option<u32>, BadInput> {
    let first = crate::menu::drivers_store::DRIVERS.first();
    for n in 1..MOST_DRIVERS {
        if crate::pointers::GamePtr::new(first + n * DRIVER_LEN).read::<u32>("a driver")?
            == u32::MAX
        {
            return Ok(Some(n));
        }
    }
    Ok(None)
}

pub fn begin_event() -> Result<(), Refused> {
    let count = drivers()?.ok_or(BadInput("the drivers do not end within 0x400"))?;
    if (0..PLAYERS).any(|n| player_at(n) == 0) {
        return Err(BadInput("a player missing from g_players").into());
    }
    MENU_PAGES.services.set(0u32);
    EVENT_STATE.stages_done.set(0u32);
    // Converted: `rc2k_core::menu::event` (`DRIVER_BEGIN`, `PLAYER_BEGIN`,
    // `KEEPS_SETUP`, `setup_defaults`).
    use rc2k_core::menu::event::{DRIVER_BEGIN, PLAYER_BEGIN, setup_defaults};
    let first = crate::menu::drivers_store::DRIVERS.first();
    crate::menu::standings::table(first, DRIVER_LEN, count)?.store_all(&DRIVER_BEGIN);
    EVENT_STATE.event_goes_on.set(1u32);
    EVENT_STATE.event_out.set(0u32);
    for n in 0..PLAYERS {
        CURRENT_PLAYER.set(n);
        let p = player_at(n);
        put_all(p, &PLAYER_BEGIN)?;
        crate::service::repair_parts::repair_car()?;
        if CHAMP_VIEW.event_mode.get() != KEEPS_SETUP {
            let sensitivity = crate::pointers::GamePtr::new(p + 0x280)
                .read::<u32>("begin event: a game value")?;
            put_all(p, &setup_defaults(sensitivity))?;
        }
    }
    Ok(())
}
