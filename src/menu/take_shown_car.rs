//! `menu::take_shown_car` (0x47B29D): the car shown on the car choice given to the current
//! player.
//!
//! In:  esi = the car list (the cars' descriptions, by handle; here a `CarList`)
//! Out: every register preserved
//!
//! The current player (`dword_75598C`) gets the entry `dword_755990` of the list: its +0 as
//! `car_number` (+8), its +8 as `no_turbo` (+0x90). Callers: `sub_482382` (after
//! `menu::select_car_loop`, esi the list it was given) and seven more. Refuses a current player
//! past the four (the original reads past `g_players`) and no list of cars.

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::{CarList, MENU_LISTS};
use crate::menu::player::{PLAYERS, Player, player};
use crate::session::Dword;

pub const CURRENT_PLAYER_RVA: &Dword = &EVENT_STATE.current_player; // dword_75598C
const _: () = assert!(std::mem::offset_of!(Player, car_number) == 8);
const _: () = assert!(std::mem::offset_of!(Player, no_turbo) == 0x90);

pub fn take_shown_car(list: Option<CarList>) -> Result<(), Refused> {
    let list = list.ok_or(BadInput("no list of cars"))?;
    let current: u32 = EVENT_STATE.current_player.get();
    if current >= PLAYERS {
        return Err(BadInput("a current player past the four").into());
    }
    let shown: u32 = EVENT_STATE.shown_car.get();
    // Converted: `rc2k_core::menu::player::Player::take_car`.
    {
        let entry = MENU_LISTS
            .cars(list)
            .offset(shown.wrapping_mul(4))
            .read::<u32>("take shown car: a game value")?;
        // The entry is a description's handle (`cars_store::car_ref`).
        let description = crate::menu::cars_store::description_ptr(entry)?;
        let number = description.read::<u32>("a car description's number")?;
        let no_turbo = description
            .offset(8)
            .read::<u32>("a car description's turbo switch")?;
        player(current)
            .ok_or(crate::errors::BadInput("a current player past the four"))?
            .update(|p| p.take_car(number, no_turbo));
    }
    Ok(())
}
