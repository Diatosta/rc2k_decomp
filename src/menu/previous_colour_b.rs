//! `menu::previous_colour_b` (0x471B7A): `dword_7554C4` one less (0 to 6) and its pick (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! A left/right handler of an options page's item. Hook removed 2026-10-22 (typed tables): the
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::menu::option_steps::{COLOUR_B, step_colour};

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    step_colour(COLOUR_B, false);
    Ok(())
}
