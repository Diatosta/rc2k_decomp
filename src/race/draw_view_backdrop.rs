//! `race::draw_view_backdrop` (0x4651E6): a quad over the view window (one pixel wider),
//! texture `dword_5D1DE8` at one place, drawn by the device's unclipped fan with the glow's
//! render states; the depth buffer on after (see `race/frame_flush.rs`).
//!
//! In:  nothing
//! Out: as the depth buffer's call leaves them
//!
//! The render states (`sub_44E44D`) and the fan (`d3d::draw_fan_unclipped`) are the
//! platform's (section 15). Caller: `sub_454806`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::frame_flush::draw_view_backdrop;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_view_backdrop(r) }?;
    Ok(())
}
