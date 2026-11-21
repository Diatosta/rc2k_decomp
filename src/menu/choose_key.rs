//! What the twelve `menu::choose_key_0` .. `_11` share: the actions of the redefine keys
//! pages' items, each starting the redefining of its key's place (`menu/redefine_keys.rs`):
//! `entry.redefining_key` = 1 and `entry.redefined_key_place` = the place.
//! `choose_key_8` is in no menu.

use crate::menu::owner::Screens;

pub(super) fn choose_key(screens: &Screens, place: u32) {
    let r = rc2k_core::menu::player::Redefining::start(place);

    screens.entry.write().redefining_key = r.on;
    screens.entry.write().redefined_key_place = r.place;
}
