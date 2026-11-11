//! `physics::record_face_contacts` (0x45E2AA): `physics::face_contact` for each face of a course
//! record (`g_courseRecords` + 0x20 n: faces +8 ..= +0xC; none when the first is -1 or past the
//! last).
//!
//! Takes: the record
//!
//! Caller: `physics::cell_face_contacts`. Refuses (before anything) a record whose faces
//! (the first not -1) run to -1 or past `g_courseFaceCount` (the original loops on or reads
//! past the faces), and what `face_contact` refuses (what was written put back).

use crate::errors::Refused;
use crate::physics::face_contact;
use crate::physics::step_families::ContactInputs;
use crate::race::owner_course::Course;
use rc2k_core::physics::ground_search;

/// The record's (first, last) faces.
pub fn record_faces(course: &Course, n: u32) -> Result<(u32, u32), Refused> {
    let rd = |offset| crate::race::course_places::record_field::<u32>(course, n, offset);
    Ok((rd(8)?, rd(0xC)?))
}

/// The steps for course record `record` , not guarded: each of its faces by `face_contact`.
// Converted: `rc2k_core::physics::ground_search::each_face`.
pub fn steps(
    c: &ContactInputs<'_>,
    record: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (first, last) = record_faces(c.course, record)?;
    ground_search::each_face(
        first,
        last,
        c.course.course.course_face_count.get(),
        |number| face_contact::steps(c, number, scene),
    )?;
    Ok(())
}
