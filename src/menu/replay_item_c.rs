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
use crate::framework::Regs;
use crate::menu::parts::{Part, parted_entry};

// race::replay_mode_2

/// The Rust version: the registers it leaves.
pub unsafe fn replay_item_c(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(0, &mut r)?;
    Ok(r)
}

fn game_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = crate::race::replay_mode_2::replay_mode_2(r, &mut crate::race::replay_mode::game_part)?;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(r, &|r, part| unsafe { replay_item_c(r, part) }, game_part)
}
