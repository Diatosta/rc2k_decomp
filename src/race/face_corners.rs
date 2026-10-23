//! `race::face_corners` (0x452444): face esi's four points into `dword_63D9EC` ..
//! `dword_63D9F8`, the fourth -1 when it is the third (see `race/course_faces.rs`).
//!
//! In:  esi = the face
//! Out: eax the fourth (a `call` follows)
//!
//! Caller: `sub_45232A`. Shadow mode: by state. Self-test: a test face, quads and
//! triangles.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::course_faces::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(crate::framework::EAX, unsafe { face_corners(r.esi) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
