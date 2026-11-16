//! `physics::face_contact` for each face of a course record (the course records: its first and last
//! face; none when the first is -1 or past the last).
//!
//! Refuses (before anything) a record whose faces (the first not -1) run to -1 or past
//! `course_face_count`, and what `face_contact` refuses (what was written is put back).

use crate::errors::Refused;
use crate::physics::face_contact;
use crate::physics::step_families::ContactInputs;
use crate::race::owner_course::Course;
use rc2k_core::physics::ground_search;

/// The record's (first, last) faces.
pub fn record_faces(course: &Course, n: u32) -> Result<(u32, u32), Refused> {
    let rd = |offset| crate::race::course_places::record_field::<u32>(course.reads(), n, offset);
    Ok((rd(8)?, rd(0xC)?))
}

/// The steps for course record `record`, not guarded: each of its faces by `face_contact`.
// Core: `rc2k_core::physics::ground_search::each_face`.
pub fn steps(
    c: &ContactInputs<'_>,
    record: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (first, last) = record_faces(c.course, record)?;
    ground_search::each_face(
        first,
        last,
        c.course.course.get().course_face_count.get(),
        |number| face_contact::steps(c, number, scene),
    )?;
    Ok(())
}
