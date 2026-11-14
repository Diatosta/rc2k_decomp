//! Finds the course record (0x20 bytes) whose first dword is the given key,
//! among its kind's records: kind = (key & 0xF) + ((key >> 11) & 0xF) * 0x10, records
//! `first_records[kind]` ..= `last_records[kind]` (`race/course_data.rs`; a first of
//! -1: none). For its callers the key is a grid cell, (z >> 21) << 11 + (x >> 21).
//!
//! Callers: `physics::ground_search` (its nine cells) and `physics::cell_face_contacts`.
//! Refuses (before anything) a kind whose last record is at or past `course_record_count`
//! while its first is not -1 (a search past the records; not worked out).

use crate::errors::Refused;
use crate::physics::cells::Trap;
use crate::race::owner_course::CourseReads;
use rc2k_core::physics::course_records::{Search, find_record, kind_of};

/// The number of the course record with `key`, if any.
pub fn find_course_record(course: CourseReads<'_>, key: u32) -> Result<Option<u32>, Refused> {
    let course_store = course;
    let (kind, _) = kind_of(key);
    // `kind_of` gives at most 0xFF: inside both tables.
    let first = course_store.kind_records.first_records[kind as usize].get();
    let last = course_store.kind_records.last_records[kind as usize].get();
    let trap = Trap::new();
    let found = find_record(
        key,
        first,
        last,
        course_store.course.course_record_count.get(),
        |n| {
            trap.keep(crate::race::course_places::record_field::<u32>(
                course, n, 0,
            ))
        },
    )?;
    trap.finish()?;
    Ok(match found {
        Search::Found { number } => Some(number),
        Search::Empty | Search::Missing { .. } => None,
    })
}
