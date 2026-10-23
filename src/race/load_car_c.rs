//! `race::load_car_c` (0x454277): `g_currentCar` the view car `dword_63DBA0`, then `drive::load_car`
//! (the car's block into the work copy).
//!
//! In:  nothing
//! Out: as `drive::load_car` leaves the registers
//!
//! See `race/frame.rs`. one caller, `sub_45B4E6`. Shadow mode: the Rust version, the work copy and
//! `g_currentCar` put back, the original's, compared. Self-test: the game's blocks, view
//! cars 0 .. 7, all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_view_car(2) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
