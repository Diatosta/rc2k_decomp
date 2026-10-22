//! `race::load_first_car` (0x460C7A): `g_currentCar` 0, `drive::load_car` (see
//! `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `drive::load_car` leaves them
//!
//! Callers: 0x429474, 0x42948D, `sub_45429B` (0x45472E), 0x459401. Refuses (before
//! anything) car 0 without its block.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_views::load_first_car;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_first_car() }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
