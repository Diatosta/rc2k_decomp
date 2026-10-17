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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::search_record_faces",
    rva(0x441FD9),
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

/// The Rust version: `r` as the original leaves it, and the carry.
pub unsafe fn search_record_faces(r: &mut Regs) -> Result<bool, Refused> {
    unsafe { search_record_faces_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    let carry = unsafe { search_record_faces(&mut g) }?;
    Ok(left_of(&g, carry))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &Ground::game().regions(), &rust);
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut test = TestGround::new(20, 25, TEST_RECORDS.len() as u32 + 1);
    let copies = match test.copies() {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    test.lay_out(2);
    set_records(&mut test, &|n| n);
    let mut found = 0;
    for (n, &(x, z)) in test_points().iter().enumerate() {
        for record in 0..TEST_RECORDS.len() as u32 {
            let prepare = || {
                test.clear_work();
                test.set_point(x, z, 0, n % 2 == 0);
            };
            if check_case(
                &mut t,
                &format!("record {record}, point ({x:#x}, {z:#x})"),
                copies.search_record_faces.addr(),
                &test,
                Regs {
                    ebx: record,
                    ..Regs::SENTINEL
                },
                &prepare,
                &|g, r| unsafe { search_record_faces_at(g, r) },
                true,
            ) {
                found += 1;
            }
        }
    }
    t.check(found > 0, || "no case found the point".into());
    // The last record (all 0x55: faces 0x55555555 ..) and one past the count.
    for record in [TEST_RECORDS.len() as u32, TEST_RECORDS.len() as u32 + 1] {
        if record < TEST_RECORDS.len() as u32 + 1 {
            let r = &mut test.records[record as usize * RECORD_LEN as usize..];
            r[8..16].fill(0x05);
        }
        let refused = unsafe {
            search_record_faces_at(
                &test.ground(),
                &mut Regs {
                    ebx: record,
                    ..Regs::SENTINEL
                },
            )
        }
        .is_err();
        t.check(refused, || format!("record {record} not refused"));
    }
    drop(copies);
    t.finish()
}
