//! `menu::replay_item_b` (0x47052F): a replay item of the review menus (see `menu/replay_items.rs`).
//!
//! In:  nothing
//! Out: the registers as its calls leave them; no flags read
//!
//! An item action in six menu tables. Hook removed 2026-10-22 (typed tables): the menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

use crate::framework::Regs;
use crate::menu::replay_items::{game_part, replay_item_b};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { replay_item_b(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
