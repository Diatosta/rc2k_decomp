//! `menu::enable_item` (0x46B50F): an item of the current menu made choosable again.
//!
//! In:  ebx = the item's number (from 1)
//! Out: every register preserved
//!
//! The item: +0x48 = 0xFFFFFC1B and its font +0x18 = the one at 0x51E8E4. The twin of
//! `menu::disable_item`, which has the details. Eight callers.
//! Refuses item 0.
//! Self-test: items of a menu of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::disable_item::set_item;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

pub fn enable_item(number: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::items::choosable`.
    set_item(number, true)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    enable_item(r.ebx)?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
