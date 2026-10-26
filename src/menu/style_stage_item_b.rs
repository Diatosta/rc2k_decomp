//! `menu::style_stage_item_b` (0x470FC3): menu item 1's look for the stage view (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::view_stage_loop`. Compared in shadow mode. Self-test:
//! `stage_view_steps::test_style`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::menu::stage_view_steps::{plain_entry, style_stage_item};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, &|_| style_stage_item(true))
}
