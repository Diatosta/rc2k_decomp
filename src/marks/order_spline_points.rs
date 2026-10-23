//! `marks::order_spline_points` (0x4BA388): record ebp's ids and points put in grade order,
//! each grade's start and count noted.
//!
//! In:  ebp the spline record
//! Out: eax +0x654, ebx `g_marksOrderedCoords`, ecx the count times 12, esi the last point
//!      taken times 12, edi 0
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (two). Only the ids and points move (the
//! two values and the grades stay: section 16), and as many as the count come back,
//! whatever their grade. Refuses more than 0x80 points or a negative count. Self-test: the
//! original and the Rust version on a record of the test's (grades 0 .. 3 by lot), the
//! refusal; the game's work tables put back afterwards.
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
    Ok(left_of(&unsafe { order_spline_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
