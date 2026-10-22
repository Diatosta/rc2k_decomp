//! `race::full_view_window_2` (0x456004): the view window the whole screen, `scene::set_view_window`,
//! `dword_63993C` 0 (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `scene::set_view_window` leaves them (eax the height before it)
//!
//! Caller: `sub_45429B`'s frame (0x454874).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::all_regs;
use crate::race::frame_views::full_view_window;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { full_view_window(r, false) }))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
