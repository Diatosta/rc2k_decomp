//! `race::draw_marks_overlays` (0x45632B): `marks::draw_overlays`, then `marks::draw_shadow`
//! (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `draw_shadow` leaves them (its caller returns)
//!
//! It is its two calls: private copy (0xB, both relocated), manual patch. Refuses (before
//! anything) the first shadow's refusals (`marks::draw_shadow` would refuse after the
//! overlays were drawn). Caller: `sub_455AD3`. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose calls go to stand-ins, against the Rust order.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_views::draw_marks_overlays;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_marks_overlays(r) }?;
    Ok(())
}
