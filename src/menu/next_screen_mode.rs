//! `menu::next_screen_mode` (0x46FFEF): the screen mode chosen one more (after the last, the first) (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: eax `dword_755648`, ebx the last mode's number
//! A left/right handler of an options page's item. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::option_steps::next_screen_mode;

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    next_screen_mode();
    Ok(())
}
