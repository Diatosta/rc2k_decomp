//! `menu::ask_to_leave`: `loop_flags.ask_leave` set: the screen loop asks whether to leave
//! (the review menus' "back to the main menu").
//!
//! An item action in six menu tables (the review menus). The menus and draw lists run its `entry`
//! (`menu::table_calls`).
use crate::menu::owner::Menus;

pub fn ask_to_leave(menus: &Menus) {
    crate::menu::leaving::with_leaving(menus, rc2k_core::menu::leaving::Leaving::ask_to_leave);
}
