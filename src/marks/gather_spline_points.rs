//! `marks::gather_spline_points` (0x4B9D5F): record ebp filled with the points of object
//! eax's faces of kind ebx.
//!
//! In:  eax the object, ebx the face kind, ebp the spline record
//! Out: eax the count, esi +0x654; ebx, ecx, edi as the last loop leaves them (see
//!      `marks/splines.rs`)
//!
//! Callers: `sub_4B9A32` (four). Refuses an object past 0xFF, a negative face count or
//! faces past `g_faces`. Self-test: the original and the Rust version on the scene of
//! `marks::measure_lines` (two objects of 0x20 faces of kinds 0 .. 3 by lot), the refusal;
//! put back afterwards.
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
    Ok(left_of(&unsafe { gather_spline_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
