//! `menu::find_player_car` (0x4708FD): the carousel turned to the current player's car.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/car_choice.rs`. Callers: `menu::select_car_loop` and
//! `menu::car_specs_47af4c_loop` at their set-up.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::car_choice::find_player_car;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { find_player_car() }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
