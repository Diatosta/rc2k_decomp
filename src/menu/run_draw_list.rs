//! `menu::run_draw_list`: `run_list` on the current screen's draw list, the one
//! `flow.draw_list` points at.
//!
//! Refuses a list without an end in its first 256 entries.

use crate::errors::Refused;
use crate::menu::run_list::run_list;
use crate::owners::Owners;

pub(super) fn run_draw_list(o: Owners<'static>) -> Result<(), Refused> {
    run_list(o, o.menus.screens.flow.draw_list.get()?)
}
