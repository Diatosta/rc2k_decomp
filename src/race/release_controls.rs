//! `race::release_controls` (0x409425): `dword_4F7DE4` 2 (the controls' state;
//! `race::create_controls` sets 3).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_444AC7` (the stage's end; `pushf` follows).
//! Shadow mode: the Rust version, the variable put back, the original's, compared.
//! Self-test: the game's variable from 0 and 3, put back.
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
    unsafe { release_controls() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
