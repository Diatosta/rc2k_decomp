//! `race::draw_lit_triangle` (0x4537E9): the triangle in entries 0 .. 2, as it is
//! (`scene::draw_clipped_polygon`) without lamp light at its corners, else cut in four at
//! its sides' middles (new points lit by `race::light_colour_at`), the colours plus the
//! lights, and each clipped and drawn (`race::clip_and_draw_face`); see
//! `race/course_faces.rs`.
//!
//! In:  nothing (entries 0 .. 2, `g_polygonVertexCount` 3)
//! Out: the registers as the parts leave them (callers return or reload)
//!
//! Callers: `sub_452CF3` (two). Refuses (before anything) no points, or no race cars.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose two drawing
//! calls go to stand-ins (projection, scaling, fade and light real), against the Rust
//! version on the same: the face stage (`race/face_stage.rs`), quiet and lit corners,
//! modes 2 and 5, views and flat projection; skipped without a race.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_lit_triangle(r) }?;
    Ok(())
}
