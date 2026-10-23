//! `race::tweak_stage` (0x439AAB): the three dwords at 0x60A564 0xFF for stages 0x15, 0x36 and 0x3E.
//!
//! In:  nothing
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { tweak_stage() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
