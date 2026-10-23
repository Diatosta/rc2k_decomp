//! `race::draw_screen_tint` (0x455A11): a tinted quad over the view window: half-through blue-grey under `dword_71C3E0`, else the sun's glare by `dword_6365F4` (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Callers: `sub_4559A6`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_views::draw_screen_tint;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_screen_tint(r) }?;
    Ok(())
}
