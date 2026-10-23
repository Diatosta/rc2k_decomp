//! `marks::turn_shape_points` (0x4BD5ED): the shape set's two lists of points turned (a
//! tilt about X, then ebx and ecx about Z) and, offset, written to their targets.
//!
//! In:  ebx, ecx the two turns (floats)
//! Out: eax the set, the rest as the last list leaves them
//!
//! See `marks/shape_points.rs`. Callers: `sub_4BD122`, `sub_4BEB25` (twice). Refuses a list
//! of more than 0x40 or an angle `fsincos` cannot take, before writing. Self-test: the
//! original and the Rust version on sets, turns and targets by lot, the refusal; put back
//! afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { turn_shape_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
