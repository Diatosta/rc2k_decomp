//! `menu::previous_car` (0x469A6F): the car specification's car before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755990` - 1, or `dword_75649C` from 0; `dword_755561` = 0;
//! `menu::load_car_background`; `sound::play_car_stream`.
//! The callback at +0x40 of the car's button (menu table at 0x74B62C), reached by the
//! arrow keys.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::car_buttons::*;

// ---- Rust version ----

pub unsafe fn previous_car(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe { press(Which::PreviousCar, call) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { previous_car(&mut |call| call_in_game(call, &entry)) }?;
    Ok(())
}
