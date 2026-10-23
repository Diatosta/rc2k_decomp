//! `race::draw_marked_face` (0x44CD90): `scene::scale_texture_coordinates`,
//! `scene::fog_polygon`, the drawing (`scene::draw_marked_polygon` with
//! `scene::draw_polygon_in_game`), `g_polygonDrawn` 1 (see `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as the parts leave them (its caller `sub_453215` returns)
//!
//! It begins with its call: private copy (0x1A, its three calls relocated), manual patch.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose drawing call
//! goes to a stand-in (the scaling and fade real), against the Rust version on the same;
//! a test quad; all put back (skipped without a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_marked_face(r, &mut marked_part) }?;
    Ok(())
}
