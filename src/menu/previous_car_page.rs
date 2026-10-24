//! `menu::previous_car_page` (0x469BAE): the car specification's page before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755998` - 1, or 3 from 0 and below (signed); `menu::load_car_background`. The callback at
//! +0x40 of the page's button (menu table at 0x74B688), reached by the arrow keys. The menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::menu::car_buttons::*;

// ---- Rust version ----

pub unsafe fn previous_car_page(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe { press(Which::PreviousPage, call) }
}

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    unsafe { previous_car_page(&mut |call| call_in_game(call)) }
}
