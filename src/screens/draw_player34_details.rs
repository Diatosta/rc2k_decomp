//! `screens::draw_player34_details` (0x467975): players 3 and 4's details page: player 1's without the second view (the
//! controller's item 8, the lines below the view 0x1B higher).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens::draw_player_details`, whose code this is (`Layout::SHORT`). In the draw lists
//! at 0x758CCA and 0x758CD2. Hook removed 2026-10-22 (typed tables): the menus and draw lists
//! run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::framework::Regs;
use crate::screens::draw_player_details::{SHORT, page_entry};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    page_entry("screens::draw_player34_details", r, SHORT)
}
