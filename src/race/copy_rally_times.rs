//! `race::copy_rally_times` (0x48C997): the rally stage's ten best times (0x9C-byte
//! records, by `dword_7558C8`, `dword_7558D0`, `dword_7558CC`) copied into the race's
//! (0x50-byte records at `dword_7556F4`), each logged.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/rally.rs`. One caller, `race::single_rally_race` (with ebx 3). Refuses,
//! before anything, a null pointer on the way. Shadow mode runs the original alone (it
//! logs). Self-test: a copy whose 50 log calls note themselves, against the Rust version
//! on the same, on the game's rally tables (skipped without them), the records filled.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::rally::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { copy_rally_times(r) }?;
    Ok(())
}
