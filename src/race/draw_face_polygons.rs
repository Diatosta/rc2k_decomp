//! `race::draw_face_polygons` (0x452CCF): in a race with both triangles facing the view
//! the face as one quad (`race::draw_whole_quad`), as triangles
//! (`race::draw_face_triangles`) otherwise or when the quad's carry asks (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as the parts leave them (its caller goes on with a call)
//!
//! Caller: `sub_452C6F`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose two calls go to stand-ins (the quad's giving each carry), against the Rust
//! version on the same; each facing; skipped without a race.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_face_polygons_in_game(r) }?;
    Ok(())
}
