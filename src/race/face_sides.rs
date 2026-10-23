//! `race::face_sides` (0x4525BE): which of face esi's triangles face the view (see
//! `race/course_faces.rs`; `sub_4525F8`, the facing test, covered here, its x87 run the
//! original's instructions): `dword_63D9E4` the first's, `dword_63D9E8` 1 or the second's.
//!
//! In:  esi = the face
//! Out: eax the last answer, ebx the last triangle's point (0 / 4 outside a race), ebp the
//!      face (a `mov` follows, not of them)
//!
//! Caller: `sub_45232A`. Shadow mode: by state. Self-test: test faces and scene points
//! (`g_scenePoints` pointed at them), normals towards and away from the view, triangles
//! and quads, in a race and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBP, EBX, ESI, Regs};
use crate::race::course_faces::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let out = unsafe { face_sides(r.esi) };
    Ok(Left::nothing()
        .with(EAX, out.eax)
        .with(EBX, out.ebx)
        .with(ESI, out.esi)
        .with(EBP, out.ebp))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
