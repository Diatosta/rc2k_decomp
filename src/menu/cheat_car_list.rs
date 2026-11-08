//! `menu::cheat_car_list` (0x480D35): with cheat bit 1 of `g_cheats` set, esi becomes the
//! cheat's car list `off_75A412`; else esi stays.
//!
//! Callers: the time trial's events, before each car choice.

use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;

/// The Rust version: the cheat cars' list (`off_75A412`) when the cheat is on.
pub fn cheat_car_list(menus: &Menus) -> Option<CarList> {
    // Converted: `rc2k_core::menu::car_lists::cheat_cars`.
    let on = rc2k_core::menu::car_lists::cheat_cars(menus.play.cheats.cheats.get());
    on.then_some(CarList::Cheat)
}
