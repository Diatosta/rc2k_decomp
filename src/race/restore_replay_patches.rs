//! `race::restore_replay_patches` (0x41F31D): see `race/replay_patches.rs`: the game's
//! own physics patches back after a replay's.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `race::close_viewed_replay`. Shadow mode compares. Self-test:
//! `replay_patches::test_patches`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay_patches::restore_replay_patches;
use crate::race::viewer::regs_entry;

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| restore_replay_patches(r))
}
