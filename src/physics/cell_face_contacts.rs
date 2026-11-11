//! `physics::cell_face_contacts` (0x45E291): `physics::record_face_contacts` for the course record
//! of the grid cell holding (x, z): the key is ((z >> 21) << 11) + (x >> 21), looked up by
//! `physics::find_course_record`; nothing when it finds none.
//!
//! Takes: x, z
//!
//! Callers: `physics::face_contacts_around` (nine). Refuses what `find_course_record`
//! (before anything) and `record_face_contacts` refuse (what was written put back).

use crate::errors::Refused;
use crate::physics::find_course_record::find_course_record;
use crate::physics::step_families::ContactInputs;

/// The steps for the cell of (`x`, `z`), not guarded.
// Converted: `rc2k_core::physics::ground_search::cell_key`.
pub fn steps(
    c: &ContactInputs<'_>,
    x: u32,
    z: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (key, _) = rc2k_core::physics::ground_search::cell_key(x, z);
    let Some(record) = find_course_record(c.course, key)? else {
        return Ok(());
    };
    crate::physics::record_face_contacts::steps(c, record, scene)
}
