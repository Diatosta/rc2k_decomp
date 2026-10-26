//! `menu::next_colour_b` (0x471B48): `dword_7554C4` one more (6 to 0) and its pick (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! A left/right handler of an options page's item. Hook removed 2026-10-22 (typed tables): the
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::option_steps::{COLOUR_B, plain_entry, step_colour};
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    step_colour(COLOUR_B, true);
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, &rust)
}
