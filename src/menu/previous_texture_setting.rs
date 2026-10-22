//! `menu::previous_texture_setting` (0x47012B): `dword_755664` one less, 0 to 2 (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: eax the value
//! A left/right handler of an options page's item. Hook removed 2026-10-22 (typed tables): the
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::option_steps::{plain_entry, step_texture_setting};
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(step_texture_setting(false))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, &rust)
}
