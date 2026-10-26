//! `menu::sort_arcade_stage_times` (0x48A0CE): the course's best times sorted up from the tenth (see `menu/arcade_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Compared in shadow mode. Self-test: `arcade_steps::test_step`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::arcade_steps::sort_arcade_stage_times;
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    sort_arcade_stage_times()?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
