//! `marks::turn_spline_points` (0x4B9FE6): record ebp's points turned by minus its three
//! angles (+0x20 about X, +0x24 about Y, +0x28 about Z).
//!
//! In:  ebp the spline record
//! Out: ecx -1, edx 0, esi and edi past the points, ebp `g_marksTurn`
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (four). Refuses more than 0x80 points or
//! an angle `fsincos` cannot take, before writing anything. Self-test: the original and
//! the Rust version on a record of the test's (0 .. 0x80 points, angles by lot), the
//! refusals; the game's matrices put back afterwards.
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
    Ok(left_of(&unsafe { turn_spline_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
