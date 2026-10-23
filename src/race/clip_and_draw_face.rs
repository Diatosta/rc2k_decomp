//! `race::clip_and_draw_face` (0x454187): the face's polygon cut by depth (when some
//! vertices are outside) and to the screen, and with three vertices or more drawn
//! (`scene::draw_polygon_in_game`), `g_polygonDrawn` 1 (see `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its callers return)
//!
//! Callers: `sub_4537E9` (four places). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose drawing call goes to a stand-in (the clips real), against the
//! Rust version on the same; test polygons inside and across the screen; all put back
//! (skipped without a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::clip_and_draw_face;

/// Its draw in the game.
pub fn draw_part(_: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { crate::scene::draw_polygon::draw_polygon_in_game(r)? };
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { clip_and_draw_face(r, &mut draw_part) }?;
    Ok(())
}
