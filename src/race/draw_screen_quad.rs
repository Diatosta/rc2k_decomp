//! `race::draw_screen_quad` (0x44B364): a flat rectangle on the screen in
//! `dword_623DE8`'s colour, from x `dword_623E10` .. `dword_623E14` and y `dword_623E20` ..
//! `dword_623E24` (see `race/frame_draw.rs`): the polygon's four work entries and
//! vertices set, clipped (`scene::clip_polygon_to_screen`), drawn with the depth buffer
//! off when three vertices or more are left.
//!
//! In:  nothing
//! Out: every register kept by the Rust version (the original's change; all four callers
//!      return at once)
//!
//! In the game the depth buffer and the drawing by `scene::depth_buffer_on`'s and
//! `scene::draw_polygon_in_game`'s Rust versions. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose depth-buffer and drawing calls go to stand-ins, the
//! clip real, against the Rust version on the same; rectangles inside, across and off the
//! screen; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_screen_quad(r, &mut quad_part) }?;
    Ok(())
}
