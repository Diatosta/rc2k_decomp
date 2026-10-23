//! `race::draw_face_sides` (0x452393): the face's front and, two-sided, its back, each
//! unless facing away (outside a race both), after `race::face_materials`; see
//! `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as it leaves them (its caller goes on with a call)
//!
//! Caller: `sub_45232A`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose two drawing calls go to stand-ins (`face_materials` real), against the Rust
//! version with the same: test faces with and without objects, one- and two-sided, each
//! facing, in a race and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_face_sides(r, &mut sides_part) }?;
    Ok(())
}
