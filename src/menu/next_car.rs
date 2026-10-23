//! `menu::next_car` (0x469AFD): the car specification's next car.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755990` + 1, or 0 when it is `dword_75649C`; `dword_755561` = 0;
//! `menu::load_car_background`; `sound::play_car_stream`.
//! Its button in the menu table at 0x74B62C.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::car_buttons::*;

// ---- Rust version ----

pub unsafe fn next_car(call: &mut dyn FnMut(Call) -> Result<(), Refused>) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe { crate::menu::car_buttons::press(crate::menu::car_buttons::Which::NextCar, call) }
}

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    unsafe { next_car(&mut |call| call_in_game(call)) }
}
