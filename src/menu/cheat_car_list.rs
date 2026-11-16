//! `menu::cheat_car_list`: with cheat bit 1 set, the shown cars list becomes the cheat car list;
//! otherwise it stays.
//!
//! Callers: the time trial's events, before each car choice.

use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;

/// The cheat cars' list (`MenuLists.cheat_cars[0]`) when the cheat is on.
pub fn cheat_car_list(menus: &Menus) -> Option<CarList> {
    let on = rc2k_core::menu::car_lists::cheat_cars(menus.play.cheats.cheats.get());
    on.then_some(CarList::Cheat)
}
