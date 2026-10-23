//! `marks::grade_spline_points` (0x4BA316): record ebp's points graded 0, 1, 2 by height.
//!
//! In:  ebp the spline record
//! Out: eax the first point's height (0x6720 with none), esi and edi 0
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (two). Refuses more than 0x80 points.
//! Self-test: the original and the Rust version on a record of the test's (0 .. 0x80
//! points), the refusal.
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
    Ok(left_of(&unsafe { grade_spline_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
