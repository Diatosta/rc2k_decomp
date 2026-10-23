//! `race::face_normal` (0x422144): the normal of three of the course's points.
//!
//! In:  eax, ebx, ecx = the points
//! Out: eax = the third's z less the first's, ebx = the third's y, ecx = its z; the rest
//!      preserved
//!
//! See `race/norms.rs`. Two callers, both in `race::store_face_normal` (inside `pusha`).
//! Self-test: points of the test's, the game's work and vector put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, Regs};
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { face_normal(r) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
