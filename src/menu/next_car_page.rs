//! `menu::next_car_page` (0x469BD1): the car specification's next page.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755998` + 1, from 3 back to 0; `menu::load_car_background`.
//! Its button in the menu table at 0x74B688.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::car_buttons::*;

// ---- Rust version ----

pub unsafe fn next_car_page(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe { crate::menu::car_buttons::press(crate::menu::car_buttons::Which::NextPage, call) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { next_car_page(&mut |call| call_in_game(call, &entry)) }?;
    Ok(())
}
