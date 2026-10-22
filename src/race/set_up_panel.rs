//! `race::set_up_panel` (0x45AB36): the panel's values each frame drawn:
//! `race::panel_values`, `panel_neighbours`, `panel_players`, `panel_motion`,
//! `panel_marks`, `both_countdown_lights`, `both_split_colours` (seven calls and a `retn`).
//!
//! In:  nothing
//! Out: the registers as the seven leave them (the caller `sub_45429B` goes on with a
//!      `call`)
//!
//! See `race/frame_draw.rs`. It begins with a call: private copy (0x24, its seven calls
//! relocated), manual patch. Refuses (before anything) what `panel_neighbours` or
//! `both_split_colours` would.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

// dword_62389C

fn rust(r: &Regs) -> Result<Left, Refused> {
    // Converted: its float callees' `_plain` versions.
    Ok(all_regs(&unsafe { set_up_panel(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
