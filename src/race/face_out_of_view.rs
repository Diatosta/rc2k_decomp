//! `race::face_out_of_view` (0x45246F): carry when none of the face's corners is in sight
//! (see `race/course_faces.rs`): each corner's scene record (made when not there), its
//! depth not negative and |x| below `flt_63DA00` or |x| x `flt_63D9FC` below the depth.
//!
//! In:  nothing (the corners `dword_63D9EC` ..)
//! Out: CF; eax the last corner looked at, edx its record (its caller tests the carry)
//!
//! Caller: `sub_45232A`. Refuses (before anything) no point records or points. Shadow
//! mode: by state and the carry. Self-test: corners on scene points of the test's
//! (`g_scenePoints` pointed at them, their records unmade), in sight and not; all put back
//! (skipped without a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EDX, Regs};
use crate::race::course_faces::face_out_of_view;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (out, carry) = unsafe { face_out_of_view(r)? };
    Ok(Left::carry(carry)
        .with(EAX, out.eax)
        .with(EDX, out.edx)
        .with(crate::framework::ECX, out.ecx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
