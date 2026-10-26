//! `menu::mark_stage_choice` (0x486C21): `dword_755700` 1 (the stage screens' map then takes
//! the choice's stage; 2 a championship's, 3 a record's).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! One caller, `menu::select_stage_loop` at its set-up.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_write};
use crate::menu::stage_choice::MODE_RVA;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

pub fn mark_stage_choice() {
    // Converted: `rc2k_core::menu::event::MapStage`.
    unsafe { game_write(MODE_RVA, rc2k_core::menu::event::MapStage::Choice as u32) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    mark_stage_choice();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
