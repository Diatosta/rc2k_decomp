//! Tries the course faces about the work car against it (`physics::face_contacts_around`).
//!
//! Refuses what `face_contacts_around` refuses (everything written is put back).

use crate::errors::Refused;
use crate::physics::face_contacts_around::face_contacts_around;
use crate::physics::step_families::ContactInputs;

pub fn face_contacts(
    c: &ContactInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    face_contacts_around(c, scene)
}
