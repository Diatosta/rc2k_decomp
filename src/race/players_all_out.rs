//! `race::players_all_out` (0x445506): carry when every player is out of the race (+0xB0
//! not 0): the chosen car with `dword_52A910` 1, else each of `dword_63DB94` players (none
//! gives no carry).
//!
//! In:  nothing
//! Out: carry set when all are out; every register preserved (`pusha` / `popa`)
//!
//! See `race/frame.rs`. Callers: `sub_444CEE` and `sub_4454D2` (stream B's; each `jb` after
//! it: dispatch sets the carry). Refuses, before anything, a chosen car past 7 or more than
//! 8 players. Shadow mode: the Rust version and the original, carry compared (nothing
//! written). Self-test: the game's blocks' +0xB0 set, one chosen car and 0 .. 8 players,
//! put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { players_all_out() }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
