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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::face_contact::{
    self, Touched, faces_region, guarded, in_race_test, move_car_onto, object_faces,
};
use crate::physics::flow::g;
use crate::screens::harness::Left;
use crate::service::harness::check_state;
use rc2k_core::physics::ground_search;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::record_face_contacts",
    rva(0x45E2AA),
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

const RECORDS: usize = 0x5CCCB8; // g_courseRecords (a pointer)

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The record's (first, last) faces.
pub fn record_faces(n: u32) -> (u32, u32) {
    let at = (n << 5).wrapping_add(g(RECORDS));
    (rd(at + 8), rd(at + 0xC))
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::ground_search::each_face`.
pub fn steps(r: &mut Regs, touched: &mut Touched) -> Result<(), Refused> {
    let (first, last) = record_faces(r.ebx);
    let record = (r.ebx << 5).wrapping_add(g(RECORDS));
    r.esi = record;
    r.ebx = ground_search::each_face(first, last, g(0x5CCD1C), |face| {
        r.ebx = face;
        face_contact::steps(r, touched)?;
        r.esi = record;
        Ok::<_, Refused>(())
    })?;
    Ok(())
}

/// The Rust version.
pub unsafe fn record_face_contacts(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, steps)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { record_face_contacts(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::physics::face_contact::dispatch_faces(&HOOK, r, call_original, steps);
}

/// The record holding face `k`, if any.
pub fn record_of(k: u32) -> Option<u32> {
    (0..g(0x5CCD18).min(0x10_0000)).find(|&n| {
        let (first, last) = record_faces(n);
        first != u32::MAX && first <= k && k <= last
    })
}

fn self_test() -> (usize, usize) {
    in_race_test(&HOOK, &mut |t, put_back| {
        let mut all = face_contact::regions();
        all.push(faces_region());
        let mut cases: Vec<(u32, Option<u32>)> = Vec::new();
        for k in object_faces(12) {
            if let Some(n) = record_of(k) {
                cases.push((n, Some(k)));
                cases.push((n, None));
            }
        }
        for n in 0..8 {
            cases.push((n, None));
        }
        for (n, moved) in cases {
            let set = || {
                put_back();
                if let Some(k) = moved {
                    move_car_onto(k);
                }
            };
            check_state(
                t,
                &format!("record {n}, car moved onto {moved:?}"),
                call_original,
                Regs {
                    ebx: n,
                    ..Regs::SENTINEL
                },
                &set,
                &all,
                &rust,
            );
        }
    })
}
