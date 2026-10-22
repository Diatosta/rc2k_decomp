//! `menu::next_screen_mode` (0x46FFEF): the screen mode chosen one more (after the last, the first) (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: eax `dword_755648`, ebx the last mode's number
//! A left/right handler of an options page's item. Hook removed 2026-10-22 (typed tables): the
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::option_steps::{next_screen_mode, plain_entry};
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(next_screen_mode())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, &rust)
}
