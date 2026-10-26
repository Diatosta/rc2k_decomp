//! `menu::lower_controller_sensitivity` (0x46B414): the controller's sensitivity one less (not below 0), set up (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! The an item action of the controller pages.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::controller_items::{item_entry, lower_controller_sensitivity};
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lower_controller_sensitivity() }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    item_entry(r, &rust)
}
