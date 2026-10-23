//! `race::free_game_map` (0x41B907): the map's two lists freed, its pointers 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/game_map.rs`. Shadow mode runs the original alone (it frees). Self-test:
//! blocks of the test's in the two lists, freed by each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::game_map::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { free_game_map() }?;
    Ok(())
}
