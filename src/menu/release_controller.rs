//! `menu::release_controller` (0x46B4B4): the input thread asked to give the devices back (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! The called on leaving the eight controller pages and by the items below. Shadow mode compares the two on the player's record, the device record
//! and the command. Self-test: `controller_items::test_item`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::controller_items::{item_entry, release_controller};
use crate::screens::harness::Left;

// The whole function is `call; retn`: the jump is written by hand and the original runs
// from a private copy with its call relocated.
fn rust(_: &Regs) -> Result<Left, Refused> {
    release_controller();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    item_entry(r, &rust)
}
