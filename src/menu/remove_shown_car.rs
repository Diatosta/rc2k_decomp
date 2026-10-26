//! `menu::remove_shown_car` (0x486BF5): the shown car taken out of its list, the numbers listed again (see `menu/arcade_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Compared in shadow mode. Self-test: `arcade_steps::test_step`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::arcade_steps::remove_shown_car;
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    remove_shown_car(&mut || unsafe {
        crate::saves::list_car_numbers::list_car_numbers().map_err(Refused::from)
    })?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
