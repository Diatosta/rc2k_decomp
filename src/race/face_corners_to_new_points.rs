//! `race::face_corners_to_new_points` (0x452722): the face's corners copied to the four
//! scene points after the last (their records marked unmade), the corners then naming the
//! copies; the count left (see `race/course_faces.rs`).
//!
//! In:  nothing (`pusha` .. `popa`)
//! Out: every register preserved
//!
//! Caller: `sub_45232A`. Refuses (before anything) no point records or points. Shadow
//! mode: by state. Self-test: test scene points (`g_scenePoints` pointed at them), quads,
//! triangles and a corner that is itself one of the new slots; all put back (skipped with
//! no point records).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::course_faces::face_corners_to_new_points;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { face_corners_to_new_points()? };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
