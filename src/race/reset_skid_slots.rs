//! `race::reset_skid_slots` (0x4585BA): a skid object's four slots (+0xF48 ..) -1.
//!
//! In:  esi = the object
//! Out: every register preserved
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { reset_skid_slots(r.esi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
