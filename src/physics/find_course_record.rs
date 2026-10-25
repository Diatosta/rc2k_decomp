//! `physics::find_course_record` (0x4220B6): the course record (0x20 bytes, `g_courseRecords`)
//! whose first dword is the key in eax, looked for among its kind's records: kind = (key &
//! 0xF) + ((key >> 11) & 0xF) * 0x10, records `g_kindFirstRecords[kind]` ..=
//! `g_kindLastRecords[kind]` (`race/course_data.rs`; -1 first: none). The key is a grid
//! cell by its callers (`sub_441FBA`: (z >> 21) << 11 + (x >> 21)).
//!
//! In:  eax the key
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
use crate::framework::game_addr;
use crate::race::course_store::COURSE;
use rc2k_core::physics::course_records::{Search, find_record, kind_of};

/// The tables' addresses and the records' count: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct RecordPlaces {
    /// `g_kindFirstRecords` (0x100 dwords; `g_kindLastRecords` 0x400 on).
    pub firsts: u32,
    pub lasts: u32,
    /// Where `g_courseRecords` (the pointer) is.
    pub records: u32,
    pub count: u32,
}

impl RecordPlaces {
    pub fn game() -> Self {
        use crate::race::course_data::KIND_TABLES_RVA;
        let first = game_addr(KIND_TABLES_RVA) as u32;
        Self {
            firsts: first,
            lasts: first + 0x400,
            records: COURSE.course_records.addr() as u32,
            count: COURSE.course_record_count.bits(),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core: the record of cell `key` (eax at entry), None when the cell has none
/// (CF). The other registers it leaves (eax, edx the key, ebp its high part, esi the
/// record's address) are read by none of its callers.
// Converted: `rc2k_core::physics::course_records::find_record`.
pub unsafe fn find_course_record_at(p: &RecordPlaces, key: u32) -> Result<Option<u32>, Refused> {
    unsafe {
        let (kind, _) = kind_of(key);
        let first = at(p.firsts + 4 * kind);
        let last = at(p.lasts + 4 * kind);
        let record = |n: u32| at(p.records).wrapping_add(n << 5);
        Ok(
            match find_record(key, first, last, p.count, |n| at(record(n)))? {
                Search::Found { number } => Some(number),
                Search::Empty | Search::Missing { .. } => None,
            },
        )
    }
}

/// The Rust version.
pub unsafe fn find_course_record(key: u32) -> Result<Option<u32>, Refused> {
    unsafe { find_course_record_at(&RecordPlaces::game(), key) }
}
