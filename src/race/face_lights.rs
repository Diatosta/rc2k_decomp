//! `race::face_lights` (0x453715): the light at each of the face's corners
//! (`dword_63DA54` .., by `race::light_colour_at`; `sub_453786` covered here), when lamps
//! are wanted and the face is not a plain mode-4 one; else all 0 (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `call` follows)
//!
//! Caller: `sub_452C6F`. Refuses (before anything) what `light_colour_at` refuses.
//! Shadow mode: by state. Self-test: the light test's scene (cars, lamps), corners of the
//! test's (points pointed at), the switches; all put back (skipped with a null block).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::course_faces::face_lights;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { face_lights(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
