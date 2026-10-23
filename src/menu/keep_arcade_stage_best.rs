//! `menu::keep_arcade_stage_best` (0x488213): the stage's best score kept in its record (see `menu/arcade_steps.rs`).
//!
//! In:  edi = the stage's record
//! Out: every register kept
//! Compared in shadow mode. Self-test: `arcade_steps::test_step`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::arcade_steps::keep_arcade_stage_best;
use crate::screens::harness::Left;

fn rust(r: &Regs) -> Result<Left, Refused> {
    keep_arcade_stage_best(r.edi)?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
