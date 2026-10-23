//! `race::seal_stage_time` (0x41F29D): the first car's time (+0x88, held to 0x22550FF;
//! its +0xB0 when that is negative) `xor` `dword_4F66F9` and turned right by its low
//! five bits, into the dword after "RCIB" (0x5BECCC); `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_41E85A` (a `mov` follows). Shadow mode:
//! the Rust version, the dword put back, the original's, compared. Self-test: times, signs
//! and keys of the test's in the game's variables, put back.
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
    unsafe { seal_stage_time() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
