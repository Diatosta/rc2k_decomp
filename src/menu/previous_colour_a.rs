//! `menu::previous_colour_a` (0x471B16): `dword_7554C0` one less (0 to 6) and its pick (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! A left/right handler of an options page's item. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::option_steps::{COLOUR_A, step_colour};

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    step_colour(COLOUR_A, false);
    Ok(())
}
