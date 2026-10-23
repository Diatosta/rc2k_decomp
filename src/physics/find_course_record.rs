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
//! `sub_45E291`.
//! Refuses (before anything) a kind whose last record is at or past `g_courseRecordCount`
//! while its first is not -1 (a search past the records; not worked out).
//! Self-test: a copy of the original on test tables and records, keys found, missing,
//! kinds empty and of one or several records.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBP, EBX, EDX, ESI, Regs, game_addr};
use crate::screens::harness::Left;
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
        use crate::race::course_data::{KIND_TABLES_RVA, RECORD_COUNT_RVA, RECORDS_RVA};
        let first = game_addr(KIND_TABLES_RVA) as u32;
        Self {
            firsts: first,
            lasts: first + 0x400,
            records: game_addr(RECORDS_RVA) as u32,
            count: unsafe { crate::framework::game_read(RECORD_COUNT_RVA) },
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core; `r` the registers at entry.
// Converted: `rc2k_core::physics::course_records::find_record`.
pub unsafe fn find_course_record_at(p: &RecordPlaces, r: &Regs) -> Result<Left, Refused> {
    unsafe {
        let key = r.eax;
        let (kind, high) = kind_of(key);
        let first = at(p.firsts + 4 * kind);
        let last = at(p.lasts + 4 * kind);
        let base = || at(p.records);
        let record = |n: u32| base().wrapping_add(n << 5);
        let left = |carry: bool| {
            Left::carry(carry)
                .with(EAX, key)
                .with(EDX, key)
                .with(EBP, high)
        };
        Ok(
            match find_record(key, first, last, p.count, |n| at(record(n)))? {
                Search::Empty => left(true).with(EBX, first),
                Search::Found { number } => left(false).with(EBX, number).with(ESI, record(number)),
                Search::Missing { next } => left(true).with(EBX, next).with(ESI, record(last)),
            },
        )
    }
}

/// The Rust version.
pub unsafe fn find_course_record(r: &Regs) -> Result<Left, Refused> {
    unsafe { find_course_record_at(&RecordPlaces::game(), r) }
}
