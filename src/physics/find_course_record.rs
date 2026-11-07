//! `physics::find_course_record` (0x4220B6): the course record (0x20 bytes, `g_courseRecords`)
//! whose first dword is the key in eax, looked for among its kind's records: kind = (key &
//! 0xF) + ((key >> 11) & 0xF) * 0x10, records `g_kindFirstRecords[kind]` ..=
//! `g_kindLastRecords[kind]` (`race/course_data.rs`; -1 first: none). The key is a grid
//! cell by its callers (`sub_441FBA`: (z >> 21) << 11 + (x >> 21)).
//!
//! Takes: the key
//! Out: CF clear when found: ebx its number, esi its address; CF set when not: ebx -1
//!      (no records) or last + 1, esi the last tried (kept when none). Always edx = eax
//!      = the key, ebp = ((key >> 11) & 0xF) << 4. ZF not read (`jb`).
//!
//! Callers (2): `sub_441FBA` (the ground's search, `sub_441E2F`'s nine cells) and
//! `sub_45E291`. Refuses (before anything) a kind whose last record is at or past
//! `g_courseRecordCount` while its first is not -1 (a search past the records; not worked
//! out).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::cells::Trap;
use crate::race::owner_course::Course;
use rc2k_core::physics::course_records::{Search, find_record, kind_of};

/// The Rust version: the number of the course record with `key`, if any.
pub fn find_course_record(course: &Course, key: u32) -> Result<Option<u32>, Refused> {
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
        course_store.course.course_record_count.bits(),
        |n| {
            trap.keep(crate::race::course_places::record_field::<u32>(
                n,
                0,
                "a course record's key",
            ))
        },
    )?;
    trap.finish()?;
    Ok(match found {
        Search::Found { number } => Some(number),
        Search::Empty | Search::Missing { .. } => None,
    })
}
