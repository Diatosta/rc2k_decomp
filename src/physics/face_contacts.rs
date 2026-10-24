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
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::face_contacts_around::face_contacts_around;

/// The Rust version.
pub fn face_contacts() -> Result<(), Refused> {
    face_contacts_around()
}
