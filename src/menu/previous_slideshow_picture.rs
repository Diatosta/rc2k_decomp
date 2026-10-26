//! `menu::previous_slideshow_picture` (0x469BF4): the slideshow's picture before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755994` - 1, 99 below 0 (signed); `menu::load_slideshow_picture` with
//! eax = -1.
//! Its button in the menu table at 0x74B85C.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::car_buttons::*;

// ---- Rust version ----

pub unsafe fn previous_slideshow_picture(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe {
        crate::menu::car_buttons::press(crate::menu::car_buttons::Which::PreviousPicture, call)
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { previous_slideshow_picture(&mut |call| call_in_game(call, &entry)) }?;
    Ok(())
}
