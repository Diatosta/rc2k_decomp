//! `menu::raise_dead_zone_a` (0x46B3A3): the first dead zone (+0x40) one more while the two leave more than 10 (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: eax the value, ebx `100 - both`, edi the player's record
//! The an item action of the controller pages.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::controller_items::{dead_zone_left, item_entry};
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    dead_zone_left(0x40)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    item_entry(r, &rust)
}
