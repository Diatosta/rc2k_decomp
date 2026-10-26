//! `screens::draw_player2_details` (0x4677B5): player 2's details page: the same code as player 1's.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens::draw_player_details`, whose code this is (`Layout::FULL`). In the draw list at
//! 0x758CC2. Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::screens::draw_player_details::{FULL, page_entry};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    page_entry("screens::draw_player2_details", r, FULL)
}
