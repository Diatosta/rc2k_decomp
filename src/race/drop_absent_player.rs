//! `race::drop_absent_player` (0x436402): in a two-player race (`dword_63DB94` 2, unless
//! `dword_62389C` is 2 or 4), the first player whose block's +0xB0 is not 0 is out of the
//! race: +0x25C 0, +0xFA8 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_cars.rs`. Callers: `sub_43603C` and its four set-ups. Shadow mode: the
//! Rust version, the blocks put back, the original's, compared. Self-test: the game's
//! blocks filled with a pattern, every mode 0 .. 5, one and two players, each player
//! absent or not, both versions, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { drop_absent_player() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
