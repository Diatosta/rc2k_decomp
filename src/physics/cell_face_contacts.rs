//! `physics::cell_face_contacts` (0x45E291): `physics::record_face_contacts` for the
//! course record of the grid cell holding (eax x, ecx z): the key ((z >> 21) << 11) +
//! (x >> 21), `physics::find_course_record`; nothing when it finds none.
//!
//! Takes: x, z
//!
//! Callers: `physics::face_contacts_around` (nine). Refuses what `find_course_record`
//! (before anything) and `record_face_contacts` refuse (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::find_course_record::find_course_record;
use crate::race::owner::Race;

/// The steps for the cell of (`x`, `z`) (eax, ecx), not guarded.
// Converted: `rc2k_core::physics::ground_search::cell_key`.
pub fn steps(
    race: &Race,
    x: u32,
    z: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (key, _) = rc2k_core::physics::ground_search::cell_key(x, z);
    let Some(record) = find_course_record(&race.course, key)? else {
        return Ok(());
    };
    crate::physics::record_face_contacts::steps(race, record, scene)
}
