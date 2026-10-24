//! `menu::replay_item_c` (0x470529): `race::replay_mode_2` and back (a 6-byte stub: `call;
//! retn`), the third replay item of the review menus (see `menu/replay_items.rs`; what it
//! shows is a reading of the viewer's mode).
//!
//! In:  nothing
//! Out: the registers as `replay_mode_2` leaves them
//!
//! An item action in the review menus' tables (0x74E5CC ..).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::screen_loop::Carried;

/// The Rust version (`c` the carried values the item gets).
pub unsafe fn replay_item_c(c: &Carried) -> Result<(), Refused> {
    crate::race::replay_mode_2::replay_mode_2(c)
}
