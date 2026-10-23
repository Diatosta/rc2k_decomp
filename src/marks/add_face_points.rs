//! `marks::add_face_points` (0x4B9E29): face edi's points added to record ebp's, each once,
//! up to 0x80.
//!
//! In:  edi the face, ebp the spline record
//! Out: eax the record's count, ecx where the last search stopped, edx the last point read
//!
//! See `marks/splines.rs`. One caller, `sub_4B9D5F`. Refuses a record of more than 0x80
//! points. Self-test: the original and the Rust version on a record and a face of the
//! test's (points repeated, the record filling up), the refusal.
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
    Ok(left_of(&unsafe { add_face_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
