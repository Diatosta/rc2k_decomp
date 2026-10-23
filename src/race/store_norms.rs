//! `race::store_norms` (0x421F02): every face's normals (`SHAUNINITEVERYTHING : store
//! norms`).
//!
//! In:  nothing
//! Out: ebx = the face count; every other register preserved
//!
//! See `race/norms.rs`. One caller, `race::loadlevel_init`. Refuses no faces or 0x100000 or
//! more. Shadow mode compares the course memory's faces, the layout and the work. Self-test:
//! courses of the test's, the game's variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, Regs};
use crate::race::norms::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// It begins with its call: the jump is written by hand and the original runs from a
// private copy, its two calls relocated.
fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { store_norms(r, &mut game_normal) }?;
    Ok(Left::nothing().with(EBX, g.ebx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
