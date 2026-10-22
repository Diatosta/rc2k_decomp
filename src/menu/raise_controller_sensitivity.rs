//! `menu::raise_controller_sensitivity` (0x46B44F): the controller's sensitivity one more (not above 3), set up (see `menu/controller_items.rs`).
//!
//! In:  nothing (edi as the caller has it: +0x2C of it is read)
//! Out: every register kept
//! The an item action of the controller pages.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::controller_items::{item_entry, raise_controller_sensitivity};
use crate::screens::harness::Left;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { raise_controller_sensitivity(r.edi) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    item_entry(r, &rust)
}
