//! `race::draw_course_face_stub` (0x4537E3): `call sub_45232A; retn`, that is
//! `race::draw_course_face`; see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: as `draw_course_face` leaves them
//!
//! It is its call: private copy (6, the call relocated), manual patch. Caller:
//! `sub_44F9F3`. Shadow mode runs the original alone (it draws). Self-test: a copy whose
//! call goes to a stand-in, against the Rust side's one call.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_course_face(r) }?;
    Ok(())
}
