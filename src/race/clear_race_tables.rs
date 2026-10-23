//! `race::clear_race_tables` (0x4632D4): entries 0 .. 9 of thirteen ten-entry tables
//! (`dword_71C010`, `71C038`, `71BFA8`, `71BFD0`, `71C088`, `71C0E8` .. `71C0F4` (four,
//! a dword apart), `71C188` .. `71C194` (four)) set to 0.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//!
//! See `race/race_cars.rs`. One caller, `race::set_up_mode_2_cars`. Shadow mode: the Rust
//! version, the tables put back, the original's, compared. Self-test: the game's tables
//! filled with a pattern, both versions, put back.
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
    unsafe { clear_race_tables() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
