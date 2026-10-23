//! `race::set_player_car_kinds` (0x43670F): the two players' blocks' +0x24, +0x2C, +0x38
//! from (`g_raceCarKinds`, `dword_6235FC`, `dword_623878`) and (`dword_62385C`,
//! `dword_623600`, `dword_62387C`).
//!
//! In:  nothing
//! Out: every register preserved (`push` / `pop` pairs)
//!
//! See `race/race_cars.rs`. Callers: `sub_43603C` and its four set-ups. Shadow mode: the
//! Rust version, the blocks put back, the original's, compared. Self-test: the game's
//! blocks filled with a pattern and kinds of the test's, both versions, put back.
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
    unsafe { set_player_car_kinds() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
