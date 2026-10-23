//! `race::store_car_b` (0x454265): `g_currentCar` the view car `dword_63DBAC`, then
//! `drive::store_car` (the work copy into the car's block).
//!
//! In:  nothing
//! Out: as `drive::store_car` leaves the registers
//!
//! See `race/frame.rs`. Callers under `sub_4541BF` (the drawing). Refuses a null block (the
//! original faults), `g_currentCar` put back. Shadow mode: the Rust version, the block and
//! `g_currentCar` put back, the original's, compared. Self-test: the game's blocks, each
//! filled with 0x5A before each run, view cars 0, 1, 5, 7 and 9; all put back.
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
    unsafe { store_view_car(1) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
