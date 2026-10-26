//! `menu::raise_dead_zone_a` (0x46B3A3): the first dead zone (+0x40) one more while the two leave more than 10 (see `menu/controller_items.rs`).
//!
//! In:  nothing
//! Out: eax the value, ebx `100 - both`, edi the player's record
//! The an item action of the controller pages. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::controller_items::raise_dead_zone;

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    raise_dead_zone(0x40)
}
