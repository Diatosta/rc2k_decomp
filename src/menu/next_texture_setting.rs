//! `menu::next_texture_setting` (0x47015B): `dword_755664` one more, 2 to 0 (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: eax the value
//! A left/right handler of an options page's item. Hook removed 2026-10-22 (typed tables): the
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::menu::option_steps::step_texture_setting;

/// The Rust version the typed tables run (what it leaves in registers is read by
/// none of its callers).
pub fn run() -> Result<(), Refused> {
    let _ = step_texture_setting(true);
    Ok(())
}
