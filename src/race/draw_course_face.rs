//! `race::draw_course_face` (0x45232A): face esi: its corners (copied to new points for
//! flag 0x200), and unless none is in sight its facings, the depth test off, its sides
//! (`race::draw_face_sides`), the depth test on; see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as it leaves them (`sub_4537E3` returns)
//!
//! It begins with its call: private copy (0x2E, its seven calls relocated), manual patch.
//! Refuses (before anything) no point records or points. Shadow mode runs the original
//! alone (it draws). Self-test: a copy whose depth-test and sides calls go to stand-ins
//! (corners, sight and facings real), against the Rust version on the same: the face
//! stage's world, test faces with and without flag 0x200, in sight and not; skipped
//! without a race.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_course_face(r, &mut course_face_part) }?;
    Ok(())
}
