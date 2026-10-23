//! `race::tidy_game_mode` (0x436B69): the game mode's tidying ("gamemodestuff tidy"):
//! nothing but, for `dword_62389C` 2, a call of `nullsub_2`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. One caller, `sub_444AC7` (`pushf` follows). Shadow mode: the
//! original and the Rust version, registers compared (nothing written). Self-test: modes
//! 0 .. 4, the game's mode put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
