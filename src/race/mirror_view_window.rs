//! `race::mirror_view_window` (0x455F17): the rear-view mirror's window (place by view and
//! table, 0xB3 x 0x40 on 800 x 600 scaled to the screen), `scene::set_view_window`,
//! `race::set_up_fog` (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `set_up_fog` leaves them (its caller in `sub_4547AE` goes on with calls)
//!
//! Refuses (before anything) a place whose scaling would fault (the original's `div`).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_views::{mirror_part, mirror_view_window};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { mirror_view_window(r, &mut mirror_part) }?;
    Ok(())
}
