//! `race::draw_back_face` (0x452B58): a two-sided face's back: the corners turned over
//! (and the first texture's order by `byte_63DA7C`), the back's colours and fog bytes,
//! drawn (`race::draw_face`), turned back; see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as it leaves them (callers return)
//!
//! Callers: `sub_452393` (two). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose `draw_face` call goes to a stand-in, against the Rust version
//! with the same: test faces, two-sided or not, triangles and quads, each order.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_back_face(r, &mut |_, r| draw_face(r, &mut face_part)) }?;
    Ok(())
}
