//! `race::draw_face` (0x452C6F): the face's textures (switches, coordinates, the mode),
//! its corners' lights, then the depth test off, the polygons
//! (`race::draw_face_polygons`), the depth test on (see `race/course_faces.rs`).
//!
//! In:  esi = the face (kept)
//! Out: esi kept; the rest as the parts leave them (callers return)
//!
//! It begins with a call: private copy (0x60, its seven calls relocated), manual patch.
//! Callers: `sub_452887`, `sub_452B58`. Refuses (before anything) no race cars when lit.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose depth-test and
//! polygon calls go to stand-ins (switches, coordinates and lights real), against the
//! Rust version on the same: the face stage, textures animated or not, a second texture,
//! lamps on and off; skipped without a race.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_face(r, &mut face_part) }?;
    Ok(())
}
