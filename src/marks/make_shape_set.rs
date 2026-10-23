//! `marks::make_shape_set` (0x4BD1A1): the next shape set made of object ebp's faces of the
//! two kinds.
//!
//! In:  ebp the object
//! Out: edi the set, the rest as `marks::find_farthest_pairs` leaves them
//!
//! See `marks/shape_points.rs`. One caller, `marks::make_shape_sets`. Refuses a set count
//! past 0x22, an object past 0xFF or faces past `g_faces`, a tilt `fsincos` cannot take,
//! before writing. Self-test: the original and the Rust version on the scene of
//! `marks::measure_lines` (faces of both kinds), 0, 1 and 0x22 sets made, tilts by lot,
//! the refusal; put back afterwards.
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
    Ok(left_of(&unsafe { make_shape_set(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
