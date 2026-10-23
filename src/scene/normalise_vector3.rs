//! `scene::normalise_vector3` (0x4654EA): the three floats at esi made one long.
//!
//! In:  esi the vector
//! Out: every register kept
//!
//! See `scene/vector3.rs`. 37 callers. Each part times 1 over the length, the original's
//! instructions (a zero vector gives NaNs, as there). Self-test: the original and the
//! Rust version on vectors by lot, zero among them.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::stage_prefs::left_of;
use crate::scene::vector3::*;
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { normalise_vector3_plain(r.esi) };
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
