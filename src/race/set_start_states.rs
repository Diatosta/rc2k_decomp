//! `race::set_start_states` (0x446D2E): the start values of the first car's kind and
//! of `dword_62385C` into `dword_6D14CA` / `CE` (the kind's record: +0x18 unless its
//! limit is past 0x1B58); `dword_6238A4` 0 or held to 1 .. 2 (two players); the
//! blocks' +0x684 2 or 3 made 4 where the kind is not flagged (`dword_5C5AC0`).
//!
//! In:  nothing
//! Out: eax = the second value, ebx = the last record offset (one player chosen: the
//!      chosen car's block); the rest preserved
//!
//! See `race/race_shell.rs`. Callers: `sub_443F0B`, `sub_443F96`, code at 0x444005,
//! `race::credits_race`, `sub_463A94` (none reads flags after it). Refuses, with
//! `dword_52A910` 1, a chosen car (`dword_52A920`) past 7, before anything. Shadow
//! mode: the Rust version, the values put back, the original's, compared. Self-test: one
//! player chosen and not, one and two players, kinds flagged and not and past the tables,
//! states 1 .. 4, the gear choice round its bounds, all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { set_start_states(r) }?;
    Ok(left_changed(r, &g))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
