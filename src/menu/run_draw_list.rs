//! `menu::run_draw_list` (0x4681ED): `run_list` on the current screen's draw list, the one
//! `flow.draw_list` points at.
//!
//! Three callers, the menu's frame step among them. Refuses a list without an end in its first 256
//! entries. Rust callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::menu::run_list::run_list;
use crate::owners::Owners;

// ---- game data ----

pub fn run_draw_list(o: Owners<'static>) -> Result<(), Refused> {
    run_list(o, o.menus.screens.flow.draw_list.get()?)
}
