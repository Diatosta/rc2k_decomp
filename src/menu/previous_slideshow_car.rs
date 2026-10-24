//! `menu::previous_slideshow_car` (0x469B35): the slideshow's car before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755990` - 1, `dword_75649C` below 0 (signed); `dword_755561` and `dword_755994` = 0;
//! `menu::load_slideshow_picture` with eax = 1; `sound::play_car_stream`. Its button in the menu
//! table at 0x74B800. The menus and draw lists run its `entry` (`menu::table_calls`); the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::car_buttons::*;

// ---- Rust version ----

pub unsafe fn previous_slideshow_car(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe {
        crate::menu::car_buttons::press(crate::menu::car_buttons::Which::PreviousSlideshowCar, call)
    }
}

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    unsafe { previous_slideshow_car(&mut |call| call_in_game(call)) }
}
