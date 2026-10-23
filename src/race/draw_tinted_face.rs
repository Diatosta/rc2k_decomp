//! `race::draw_tinted_face` (0x44F9F3): face esi drawn (`race::draw_course_face`) with
//! `g_polygonColour` times its object's tint (`dword_60A880`), put back after; see
//! `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as the drawing leaves them (eax the tinted colour before it)
//!
//! Caller: `race::draw_list_entry` (place 2). Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose drawing call goes to a stand-in that notes the colour,
//! against the Rust version with the same: tints 0 and not, MMX on and off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_tinted_face(r, &mut tinted_part) }?;
    Ok(())
}
