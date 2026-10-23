//! `race::draw_whole_quad` (0x452F3D): the face as one quad (entries 1, 2, 3, 0 from
//! corners 0, 1, 3, 2) drawn by `race::draw_marked_face` when no corner has lamp light
//! and all vertices are inside the depths and the screen; carry asks for triangles (see
//! `race/course_faces.rs`).
//!
//! In:  nothing (the face's working values)
//! Out: CF; the other registers as it leaves them (its caller tests the carry)
//!
//! Caller: `sub_452CCF`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose drawing call goes to a stand-in (the projection real), against the Rust
//! version on the same: the face stage, quiet and lit corners, views (inside, partly
//! outside), the carry compared; skipped without a race.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_whole_quad(r, &mut whole_quad_part).map(|carry| r.set_carry(carry)) }?;
    Ok(())
}
