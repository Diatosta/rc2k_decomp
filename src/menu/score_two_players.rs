//! `menu::score_two_players` (0x485B0E): the two players' places, points, bonuses and totals (see `menu/results_setups.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::results_48584b_loop` (set-up). Compared in shadow mode. Self-test: `results_setups::test_results`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::menu::results_setups::score_two_players;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &score_two_players)
}
