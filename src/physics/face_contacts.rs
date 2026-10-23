//! `physics::face_contacts` (0x45E190): `physics::face_contacts_around` (its only code,
//! `call` and `retn`): the course faces about the work car tried against it.
//!
//! In:  nothing
//! Out: the registers as `face_contacts_around` leaves them; no flags read
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`. The hook's five bytes are
//! the `call`: a private copy (six bytes, the call relocated) and a hand-made patch, as
//! `scene::end_scene`.
//! Refuses what `face_contacts_around` refuses (everything written put back).
//! Self-test: `face_contacts_around`'s (in a race; skipped otherwise).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::face_contacts_around::face_contacts_around;
use crate::screens::harness::Left;

/// The Rust version.
pub unsafe fn face_contacts(entry: &Regs) -> Result<Left, Refused> {
    unsafe { face_contacts_around(entry) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::physics::face_contact::faces_entry(r, crate::physics::face_contacts_around::steps)
}
