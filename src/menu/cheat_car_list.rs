//! `menu::cheat_car_list` (0x480D35): with cheat bit 1 of `g_cheats` set, esi becomes the
//! cheat's car list `off_75A412`; else esi stays.
//!
//! In:  esi a car list
//! Out: esi; carry = the bit (`bt`); the rest kept
//!
//! Callers: the time trial's events, before each car choice.

use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::lists_store::MENU_LISTS;

/// The Rust version: the cheat cars' list (`off_75A412`) when the cheat is on.
pub fn cheat_car_list() -> Option<u32> {
    // Converted: `rc2k_core::menu::car_lists::cheat_cars`.
    let on = rc2k_core::menu::car_lists::cheat_cars(CHEAT_VARS.cheats.get());
    on.then(|| MENU_LISTS.cheat_cars[0].addr())
}

/// The registers the original leaves for `list`, for register-threaded callers: esi the
/// list when there is one, the carry whether there is.
pub fn carry_on(list: Option<u32>, c: &mut crate::menu::screen_loop::Carried) {
    if let Some(list) = list {
        c.cars = list;
    }
    c.carry = list.is_some();
}
