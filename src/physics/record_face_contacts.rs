//! `physics::record_face_contacts` (0x45E2AA): `physics::face_contact` for each face of
//! the course record ebx (`g_courseRecords` + 0x20 n: faces +8 ..= +0xC; none when the
//! first is -1 or past the last), esi and ebx kept around each call.
//!
//! In:  ebx the record
//! Out: esi the record, ebx the last face + 1 (or the first, when none), the rest as the
//!      calls leave them; no flags read (the caller returns)
//!
//! Caller: `physics::cell_face_contacts`. Refuses (before anything) a record whose faces
//! (the first not -1) run to -1 or past `g_courseFaceCount` (the original loops on or reads
//! past the faces), and what `face_contact` refuses (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::face_contact;
use crate::race::course_store::COURSE;
use rc2k_core::physics::ground_search;

fn rd(x: u32) -> Result<u32, Refused> {
    Ok(crate::race::course_places::read::<u32>(
        x,
        "a course record's face number",
    )?)
}

/// The record's (first, last) faces.
pub fn record_faces(n: u32) -> Result<(u32, u32), Refused> {
    let at = (n << 5).wrapping_add(COURSE.course_records.get());
    Ok((rd(at + 8)?, rd(at + 0xC)?))
}

/// The steps for course record `record` (ebx), not guarded: each of its faces by
/// `face_contact`. The registers left are read by none of the callers.
// Converted: `rc2k_core::physics::ground_search::each_face`.
pub fn steps(record: u32) -> Result<(), Refused> {
    let (first, last) = record_faces(record)?;
    ground_search::each_face(
        first,
        last,
        COURSE.course_face_count.get(),
        face_contact::steps,
    )?;
    Ok(())
}
