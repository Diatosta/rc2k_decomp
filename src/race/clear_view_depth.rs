//! `race::clear_view_depth` (0x44E9A0): with the device on, the clip rectangle the whole screen and the depth cleared there (`sub_44E9EE`, the platform's) (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller goes on with calls)
//!
//! Callers: `race::draw_own_car`. Shadow mode runs the original alone (it clears the depth buffer). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version with the same: the device on and off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_views::{clear_view_depth, own_car_clear_part};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { clear_view_depth(r, &mut own_car_clear_part) }?;
    Ok(())
}
