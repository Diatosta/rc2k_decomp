//! `physics::cell_face_contacts` (0x45E291): `physics::record_face_contacts` for the
//! course record of the grid cell holding (eax x, ecx z): the key ((z >> 21) << 11) +
//! (x >> 21), `physics::find_course_record`; nothing when it finds none.
//!
//! In:  eax x, ecx z
//! Out: the registers as `find_course_record` and `record_face_contacts` leave them; no
//!      flags read (the caller goes on)
//!
//! Callers: `physics::face_contacts_around` (nine).
//! Refuses what `find_course_record` (before anything) and `record_face_contacts`
//! refuse (what was written put back).
//! Self-test (in a race; skipped otherwise): the original and the Rust version on the
//! stage (everything written saved and put back): the nine cells about the car, where it
//! is and moved onto object faces.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::face_contact::Touched;
use crate::physics::find_course_record::find_course_record;

/// The steps, not guarded.
// Converted: `rc2k_core::physics::ground_search::cell_key`.
pub fn steps(r: &mut Regs, touched: &mut Touched) -> Result<(), Refused> {
    (r.eax, r.ecx) = rc2k_core::physics::ground_search::cell_key(r.eax, r.ecx);
    let found = unsafe { find_course_record(r) }?;
    found.apply(r);
    if found.carry.unwrap_or(false) {
        return Ok(());
    }
    crate::physics::record_face_contacts::steps(r, touched)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::physics::face_contact::faces_entry(r, steps)
}
