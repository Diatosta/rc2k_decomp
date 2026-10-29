//! `menu::next_car_page` (0x469BD1): the car specification's next page.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755998` + 1, from 3 back to 0; `menu::load_car_background`. Its button in the menu table
//! at 0x74B688. The menus and draw lists run its `entry` (`menu::table_calls`); the registers above
//! are the original's, for reference.

use crate::errors::Refused;
use crate::menu::car_buttons::*;

// ---- Rust version ----

pub fn next_car_page(call: &mut dyn FnMut(Call) -> Result<(), Refused>) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    crate::menu::car_buttons::press(crate::menu::car_buttons::Which::NextPage, call)
}

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    next_car_page(&mut |call| call_in_game(call))
}
