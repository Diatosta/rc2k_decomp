//! `race::face_depth_test_off` (0x452358): with no texture animation (`dword_63DA2C` 0)
//! and `g_polygonMode` 4, `scene::depth_test_off` and `dword_63D9E0` 1 (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: as the depth test's call leaves them (eax 0, ebx 0xE), or all kept
//!
//! Caller: `sub_452C6F`, `sub_45232A`. Shadow mode runs the original alone (it tells the
//! device). Self-test: a copy whose depth-test call goes to a stand-in, against the Rust
//! version with the same; the two switches.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_faces::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { face_depth_test_off(r, &mut depth_part) }?;
    Ok(())
}
