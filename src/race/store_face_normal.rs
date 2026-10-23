//! `race::store_face_normal` (0x421F72): a face's repeated-points flag and its two
//! normals.
//!
//! In:  ebx = the face
//! Out: every register preserved
//!
//! See `race/norms.rs`. One caller, `race::store_norms`. Self-test: a course of the test's
//! (`norms::TestCourse`, three and four points, points out of range and repeated), every
//! face, the game's variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { store_face_normal(r.ebx, &mut game_normal) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
