//! `race::load_car_a` (0x45422F): `g_currentCar` the view car `dword_63DBA8`, then `drive::load_car`
//! (the car's block into the work copy).
//!
//! In:  nothing
//! Out: as `drive::load_car` leaves the registers
//!
//! See `race/frame.rs`. 13 callers in `sub_45429B` and `sub_4544DD`.. (the drawing).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_view_car(0) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
