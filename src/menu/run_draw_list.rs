//! `menu::run_draw_list` (0x4681ED): `run_list` on the current screen's draw list, the one
//! `dword_7554F8` points at.
//!
//! Three callers, the menu's frame step among them. Refuses a list without an end in its first 256
//! entries. Rust callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::run_list::run_list;

// ---- game data ----

// ---- Rust version ----

pub fn run_draw_list(menus: &Menus) -> Result<(), Refused> {
    run_list(menus, menus.screens.flow.draw_list.get()?)
}
