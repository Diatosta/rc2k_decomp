//! `menu::next_stage_rows_of_90` (0x4A37B5): the event page's rows on by ten (see `menu/stage_rows.rs`).
//!
//! In:  nothing
//! Out: eax as `stage_rows` says; the rest kept
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::stage_rows::rows_entry;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    rows_entry(r, true, true)
}
