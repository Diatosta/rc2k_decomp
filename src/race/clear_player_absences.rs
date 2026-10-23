//! `race::clear_player_absences` (0x449D44): the eight car blocks' +0xB0 (the player
//! absent, `race::drop_absent_player`'s test) 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_442FEF`, `sub_44306B`, `sub_443104` and
//! `sub_446EBE` (none reads flags after it). Shadow mode: the Rust version, the blocks put
//! back, the original's, compared. Self-test: the game's eight values set, put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_shell::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_player_absences() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
