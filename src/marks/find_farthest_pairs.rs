//! `marks::find_farthest_pairs` (0x4BDD1D): of each points list, the two points of one face
//! farthest apart.
//!
//! In:  nothing (points lists A and B, `g_marksTurnedA` / `B`)
//! Out: as the last pair looked at leaves them (edx the farthest of B)
//!
//! See `marks/shape_points.rs`. One caller, `sub_4BD1A1`. Refuses a list of more than 0x40.
//! Self-test: the original and the Rust version on lists and points by lot (0 .. 0x40, a
//! few faces), the refusal; put back afterwards.
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
    Ok(left_of(&unsafe { find_farthest_pairs(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
