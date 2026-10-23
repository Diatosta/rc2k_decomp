//! `menu::count_stage_pages` (0x480032): how many pages the stage has (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::view_stage_loop`. Compared in shadow mode. Self-test: records of the
//! test's at the start of `g_recordTable`, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::stage_view_steps::{count_stage_pages, plain_entry};
use crate::screens::harness::Left;

fn rust(_: &Regs) -> Result<Left, Refused> {
    count_stage_pages()?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, &rust)
}
