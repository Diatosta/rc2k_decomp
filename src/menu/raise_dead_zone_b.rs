//! `menu::raise_dead_zone_b` (0x46B3EA): the second dead zone (+0x44) one more while the two leave more than 10 (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! The an item action of the controller pages.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::controller_items::dead_zone_left;

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    dead_zone_left(0x44).map(|_| ())
}
