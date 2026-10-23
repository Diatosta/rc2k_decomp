//! `physics::search_record_faces` (0x441FD9): the faces +8 ..= +0xC of course record ebx
//! (`g_courseRecords`, 0x20 bytes), each by `physics::face_under_point` (ebx and esi
//! pushed around it), until one has the point: CF set. CF clear when none does, or the
//! first is past the last (unsigned) or -1.
//!
//! In:  ebx the record
//! Out: CF; esi the record's address, ebx the face found (or one past the last, or the
//!      first when none was tried); eax, ecx, ebp, edi as the last `face_under_point`
//!      left them; edx kept
//!
//! See `physics/ground.rs`. Caller: `physics::search_cell` (`jb`).
//! Refuses (before anything) a record at or past `g_courseRecordCount`, and a record whose
//! faces run past `g_courseFaceCount`.
//! Self-test: a copy of the original (its call to a copy of `face_under_point`) on the
//! test's ground: records of no faces, one, all sixteen squares, a range turned round and
//! one of -1, against the test's points; the refusals.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: `r` as the original leaves it, and the carry.
pub unsafe fn search_record_faces(r: &mut Regs) -> Result<bool, Refused> {
    unsafe { search_record_faces_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    let carry = unsafe { search_record_faces(&mut g) }?;
    Ok(left_of(&g, carry))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// Records for the tests: (first, last) faces.
pub const TEST_RECORDS: [(u32, u32); 7] = [
    (0, 15),
    (5, 5),
    (3, 9),
    (9, 3),
    (u32::MAX, u32::MAX),
    (12, 19),
    (0, 0),
];

/// Puts `TEST_RECORDS` into the test's records, record n's key `key(n)`.
pub fn set_records(test: &mut TestGround, key: &dyn Fn(u32) -> u32) {
    for (n, &(first, last)) in TEST_RECORDS.iter().enumerate() {
        let r = &mut test.records[n * RECORD_LEN as usize..(n + 1) * RECORD_LEN as usize];
        r.fill(0x55);
        r[0..4].copy_from_slice(&key(n as u32).to_le_bytes());
        r[8..12].copy_from_slice(&first.to_le_bytes());
        r[12..16].copy_from_slice(&last.to_le_bytes());
    }
}
