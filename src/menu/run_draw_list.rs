//! `menu::run_draw_list` (0x4681ED): `run_list` on the current screen's draw list, the one
//! `dword_7554F8` points at.
//!
//! In:  nothing
//! Out: every register preserved
//! Three callers, the menu's frame step among them. Refuses a list without an end in its first 256
//! entries. Rust callers run its `entry` (`routes.rs`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::run_list::run_list;

// ---- game data ----

// ---- Rust version ----

pub fn run_draw_list() -> Result<(), Refused> {
    run_list(SCREEN_FLOW.draw_list.get())
}
