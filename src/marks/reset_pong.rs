//! `marks::reset_pong` (0x4B7E64): the pong game put back to its start: four (x, y) pairs
//! (the ball and the bats) set from `flt_769182` .., the ball's steps (`g_pongSteps`,
//! 0x768700) 5, the hits (`g_pongHits`) 0. Named `reset_outline` until 2026-10-18.
//!
//! In:  nothing
//! Out: every register kept
//!
//! See `marks/splines.rs`. Callers: `sub_4B7F4F` and `sub_4BF529`. Self-test:
//! the original and the Rust version with the eight floats by lot (a signalling NaN among
//! them, which `fld` makes quiet), put back afterwards.
//!
//! Converted 2026-10-21: through `rc2k_core::marks::pong::Court::reset` (exact at every
//! precision); compared exactly as before.
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

// flt_769182

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { reset_pong() };
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
