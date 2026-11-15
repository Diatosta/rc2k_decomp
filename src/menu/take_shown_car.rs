//! `menu::take_shown_car`: the car shown on the car choice given to the current player.
//!
//! The current player (`event.current_player`) gets the entry `event.shown.car` of the list: its
//! +0 as `car_number` (+8), its +8 as `class` (+0x90). Callers: seven more. Refuses a current
//! player past the four (it would read past the players) and no list of cars.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::lists;
use crate::menu::lists_store::{CarList, CarListParts};
use crate::menu::owner::Menus;
use crate::menu::player::{PLAYERS, player};

pub fn take_shown_car(menus: &Menus, list: Option<CarList>) -> Result<(), Refused> {
    let list = list.ok_or(BadInput("no list of cars"))?;
    let current: u32 = menus.screens.event.current_player.get();
    if current >= PLAYERS {
        return Err(BadInput("a current player past the four").into());
    }
    let shown: u32 = menus.screens.event.shown.car.get();
    {
        let entry = lists().car_entry(CarListParts::of(menus), list, shown)?;
        // The entry is a description's handle (`cars_store::car_ref`).
        let description = crate::menu::cars_store::description(&menus.cars.table, entry)?.get();
        let (number, class) = (description.car, description.class);
        player(menus, current)
            .ok_or(crate::errors::BadInput("a current player past the four"))?
            .update(|p| p.take_car(number, class));
    }
    Ok(())
}
