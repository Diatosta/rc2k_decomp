//! `race::play_from_here` (0x41E1E2): see `race/play_marked.rs`: the viewer plays from the
//! block and frame shown to the end.
//!
//! In:  nothing
//! Out: eax the block played from, ebx and ecx its frame; no flags read
//!
//! Callers: `race::viewer_action` (6, and 0x22, 0x23 after it). Ungated (it plays a
//! replay). Shadow mode runs the original alone. Self-test: `play_marked::test_play` from
//! the shared tail.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::play_marked::{game_part, play_from_here};
use crate::race::viewer::regs_entry;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| play_from_here(r, &mut game_part))
}
