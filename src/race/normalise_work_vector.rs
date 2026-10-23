//! `race::normalise_work_vector` (0x41FC4D): `g_workVector` made one long.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/norms.rs` (the length is the 80-bit root, unlike `scene::unit_vector`'s). Eight
//! callers (`race::face_normal` and the race's code). Self-test: the game's vector and
//! length with vectors of the test's, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { normalise_work_vector() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
