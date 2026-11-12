//! `menu::take_shown_car` (0x47B29D): the car shown on the car choice given to the current
//! player.
//!
//! The current player (`event.current_player`) gets the entry `event.shown_car` of the list: its +0 as
//! `car_number` (+8), its +8 as `class` (+0x90). Callers: seven more. Refuses a current player
//! past the four (the original reads past `g_players`) and no list of cars.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::player::{PLAYERS, Player, player};

const _: () = assert!(std::mem::offset_of!(Player, car_number) == 8);
const _: () = assert!(std::mem::offset_of!(Player, class) == 0x90);

pub fn take_shown_car(menus: &Menus, list: Option<CarList>) -> Result<(), Refused> {
    let list = list.ok_or(BadInput("no list of cars"))?;
    let current: u32 = menus.screens.event.current_player.get();
    if current >= PLAYERS {
        return Err(BadInput("a current player past the four").into());
    }
    let shown: u32 = menus.screens.event.shown_car.get();
    // The logic is in `rc2k_core::menu::player::Player::take_car`.
    {
        let entry = menus.tables.lists.car_entry(list, shown)?;
        // The entry is a description's handle (`cars_store::car_ref`).
        let description = crate::menu::cars_store::description(entry)?.get();
        let (number, class) = (description.car, description.class);
        player(menus, current)
            .ok_or(crate::errors::BadInput("a current player past the four"))?
            .update(|p| p.take_car(number, class));
    }
    Ok(())
}
