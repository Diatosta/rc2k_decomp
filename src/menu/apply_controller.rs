//! `menu::apply_controller` (0x46B47C): the current player's controller asked of the input thread, if it is a listed one (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! The set-up of the eight controller pages (`Step::Call` in their loops) and called by the items below. Shadow mode compares the two on the player's record, the device record
//! and the command. Self-test: `controller_items::test_item`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::controller_items::{apply_controller, item_entry};
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_controller() }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    item_entry(r, &rust)
}
