//! `race::face_depth_test_on` (0x45237A): `scene::depth_test_on` unless `dword_63D9E0` is
//! set, then `dword_63D9E0` 0 (see `race/course_faces.rs`; the test looks the wrong way
//! round, `notes/bugs.md`).
//!
//! In:  nothing
//! Out: as the depth test's call leaves them (eax 1, ebx 0xE), or all kept
//!
//! Callers: `sub_452C6F`, `sub_45232A`. Shadow mode runs the original alone (it tells the
//! device). Self-test: as `face_depth_test_off`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { face_depth_test_on(r) }?;
    Ok(())
}
