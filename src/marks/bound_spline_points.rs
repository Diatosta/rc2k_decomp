//! `marks::bound_spline_points` (0x4B9E7D): record ebp's points' box: its middle made the
//! centre, its angle from `g_marksSplineUp` into +0x20.
//!
//! In:  ebp the spline record
//! Out: eax, ebx, ecx the middle, esi `g_marksSplineUp`, edi `g_marksSplineSize`
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (four). Refuses more than 0x80 points.
//! Self-test: the original and the Rust version on a record of the test's (0 .. 0x80
//! points) with the two vectors by lot, the refusal; the game's vectors put back
//! afterwards.
//!
//! Converted 2026-10-21: the box through `rc2k_core::marks::splines::bounds`; the sizes
//! made floats and the angle stay the x87's. Compared exactly as before.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { bound_spline_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
