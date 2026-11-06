//! `physics::face_contacts` (0x45E190): `physics::face_contacts_around` (its only code,
//! `call` and `retn`): the course faces about the work car tried against it.
//!
//! In:  nothing
//! Out: the registers as `face_contacts_around` leaves them; no flags read
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`. Refuses what
//! `face_contacts_around` refuses (everything written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::face_contacts_around::face_contacts_around;
use crate::race::owner::Race;

/// The Rust version.
pub fn face_contacts(race: &Race) -> Result<(), Refused> {
    face_contacts_around(race)
}
