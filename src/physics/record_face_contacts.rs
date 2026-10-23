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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::face_contact::{self, Touched, guarded};
use crate::physics::flow::g;
use crate::screens::harness::Left;
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::physics::face_contact::faces_entry(r, steps)
}

/// The record holding face `k`, if any.
pub fn record_of(k: u32) -> Option<u32> {
    (0..g(0x5CCD18).min(0x10_0000)).find(|&n| {
        let (first, last) = record_faces(n);
        first != u32::MAX && first <= k && k <= last
    })
}
