//! `race::shadow_polygon` (0x45787F): the shadow's hull as scene points (after the scene's
//! own, from `g_scenePointCount` + 0x10) and as the polygon's work entries with a circle's
//! texture places (see `race/car_shadow.rs`; x87 the original's instructions).
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `mov` follows, not of them)
//!
//! Refuses (before anything) a hull of no points or more than 0x40. Shadow mode: by state
//! (the work entries, the new scene points). Self-test: hulls of 3, 5 and 8 records of
//! the test's, scene points of the test's (`g_scenePoints` pointed at them); all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::car_shadow::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { shadow_polygon(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
