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

use crate::errors::Refused;
use crate::framework::{
    EAX, EBP, EBX, EDX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::course_records::{Search, find_record, kind_of};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::find_course_record",
    rva(0x4220B6),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { find_course_record(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const RECORDS: u32 = 0x40;
    // firsts 0x100, lasts 0x100, the records' pointer, records 0x40 x 8 dwords.
    let mut cells = vec![0u32; 0x200 + 1 + 8 * RECORDS as usize];
    let base = cells.as_mut_ptr() as u32;
    let records = base + 0x804;
    let p = RecordPlaces {
        firsts: base,
        lasts: base + 0x400,
        records: base + 0x800,
        count: RECORDS,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4220B6,
            len: 0x41,
            entry: &[0x8B, 0xD0, 0x8B, 0xC2, 0x83, 0xE0, 0x0F],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x5CC3E4, p.firsts),
            (0x5CC7E4, p.lasts),
            (0x5CCCB8, p.records),
        ],
        3,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    // Kinds: 0 empty; 1 one record; 0x23 four; 0xFF three. Record n's key: a key of its
    // kind (cell numbers chosen to land there).
    let key_of = |kind: u32, k: u32| (kind & 0xF) | ((kind >> 4) << 11) | (k << 4) | (k << 15);
    let layout: [(u32, u32, u32); 3] = [(1, 0, 0), (0x23, 1, 4), (0xFF, 5, 7)];
    unsafe {
        let c = base as *mut u32;
        for k in 0..0x100 {
            c.add(k).write(u32::MAX);
            c.add(0x100 + k).write(0);
        }
        c.add(0x200).write(records);
        for n in 0..RECORDS {
            for d in 0..8 {
                c.add(0x201 + 8 * n as usize + d)
                    .write(0xBB00_0000 | n << 4 | d as u32);
            }
        }
        for (kind, first, last) in layout {
            c.add(kind as usize).write(first);
            c.add(0x100 + kind as usize).write(last);
            for n in first..=last {
                c.add(0x201 + 8 * n as usize).write(key_of(kind, n - first));
            }
        }
    }
    let mut keys = vec![key_of(0, 0), key_of(0, 3), 0xFFFF_FFFF, 0x8000_0010];
    for (kind, first, last) in layout {
        for k in 0..=(last - first + 1) {
            keys.push(key_of(kind, k));
        }
    }
    for key in keys {
        let before = Regs {
            eax: key,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let mine = unsafe { find_course_record_at(&p, &before) };
        let ok = mine.as_ref().is_ok_and(|l| l.matches(&before, &ro));
        t.check(ok, || {
            format!(
                "key {key:#x}: original ebx {:#x} esi {:#x} ebp {:#x} carry {}; rust {mine:?}",
                ro.ebx,
                ro.esi,
                ro.ebp,
                ro.carry()
            )
        });
    }
    // A kind running past the count is refused.
    unsafe { (base as *mut u32).add(0x100 + 0x23).write(RECORDS) };
    let refused = unsafe {
        find_course_record_at(
            &p,
            &Regs {
                eax: key_of(0x23, 9),
                ..Regs::SENTINEL
            },
        )
    }
    .is_err();
    t.check(refused, || "a kind past the count not refused".into());
    drop((copy, cells));
    t.finish()
}
