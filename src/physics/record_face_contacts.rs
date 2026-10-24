//! `physics::record_face_contacts` (0x45E2AA): `physics::face_contact` for each face of
//! the course record ebx (`g_courseRecords` + 0x20 n: faces +8 ..= +0xC; none when the
//! first is -1 or past the last), esi and ebx kept around each call.
//!
//! In:  ebx the record
//! Out: esi the record, ebx the last face + 1 (or the first, when none), the rest as the
//!      calls leave them; no flags read (the caller returns)
//!
//! Caller: `physics::cell_face_contacts`.
//! Refuses (before anything) a record whose faces (the first not -1) run to -1 or past `g_courseFaceCount`
//! (the original loops on or reads past the faces), and what `face_contact` refuses (what
//! was written put back).
//! Self-test (in a race; skipped otherwise): the original and the Rust version on the
//! stage (everything written saved and put back): the records of the car's cell where it
//! is and of object faces with the car moved onto them.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::face_contact;
use crate::physics::flow::g;
use rc2k_core::physics::ground_search;

const RECORDS: usize = 0x5CCCB8; // g_courseRecords (a pointer)

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The record's (first, last) faces.
pub fn record_faces(n: u32) -> (u32, u32) {
    let at = (n << 5).wrapping_add(g(RECORDS));
    (rd(at + 8), rd(at + 0xC))
}

/// The steps for course record `record` (ebx), not guarded: each of its faces by
/// `face_contact`. The registers left are read by none of the callers.
// Converted: `rc2k_core::physics::ground_search::each_face`.
pub fn steps(record: u32) -> Result<(), Refused> {
    let (first, last) = record_faces(record);
    ground_search::each_face(first, last, g(0x5CCD1C), face_contact::steps)?;
    Ok(())
}
