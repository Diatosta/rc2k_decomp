//! `menu::previous_choice_car` (0x47CED5): the car choice's carousel turned to the previous car.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/car_choice.rs`. The car choice screens' item actions (three tables).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::car_choice::previous_choice_car;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { previous_choice_car() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
