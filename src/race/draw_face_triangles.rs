//! `race::draw_face_triangles` (0x452CF3): the face's two triangles (corners 0, 1, 2
//! and 3, 2, 1) into entries 0 .. 2 and each drawn (`race::draw_lit_triangle`) unless
//! facing away (see `race/course_faces.rs`).
//!
//! In:  nothing (the face's working values)
//! Out: the registers as the parts leave them (its caller returns)
//!
//! Caller: `sub_452CCF`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose two drawing calls go to a stand-in, against the Rust version on the same:
//! the face stage, each facing, triangles and quads; skipped without a race.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_face_triangles(r, &mut triangles_part) }?;
    Ok(())
}
