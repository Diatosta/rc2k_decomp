//! `race::face_colours` (0x452887): the corners' colours (`g_lightTable` by the face's
//! +0x14 .., with flag 4 plus the two extra lights, `add_colours` MMX or not) and fog
//! bytes, then the face drawn (`race::draw_face`); see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as `draw_face` leaves them (callers return or go on with `xor`)
//!
//! Callers: `sub_452393` (three). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose `draw_face` call goes to a stand-in, against the Rust version
//! with the same: test faces, flag 4 on and off, MMX on and off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { face_colours(r, &mut |_, r| draw_face(r)) }?;
    Ok(())
}
