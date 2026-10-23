//! `marks::find_farthest_points` (0x4BA578): the two farthest apart of eax points at esi,
//! looked for inside a `pusha` .. `popa` and so kept nowhere (section 16).
//!
//! In:  eax the count, esi the points (12 bytes)
//! Out: every register kept, nothing written
//!
//! See `marks/splines.rs`. One caller, `sub_4B9D5F`. Self-test: the original and the Rust
//! version (which does nothing) on a record of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&find_farthest_points(r)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
