//! `marks::collect_points_b` (0x4BD59F): face edi's points added to points list B
//! (`g_marksPointsB`), each once with the face, up to 0x40.
//!
//! In:  edi the face
//! Out: eax the count, ebx the last point read, ecx where the last search stopped, ebp past
//!      the last point taken in
//!
//! See `marks/masks.rs`. One caller, `sub_4BD1A1` (0x4BD2A5). Refuses a list of more than
//! 0x40. Self-test: the original and the Rust version on a face of the test's (points
//! repeated, the list filling up), the refusal; the list put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { collect_points(r, &POINTS_B) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
