//! `physics::clear_forces` (0x43F395): `physics::clear_records` on 0x17 records of 0x20 bytes at `unk_614770` (by the look of
//! it the step's forces, a reading).
//!
//! In:  nothing
//! Out: edi past the records, ecx 0; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B71E), every physics step.
//! Self-test: a copy of the original (its records a test buffer, its call kept) against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::clear_records::clear_records;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const RECORDS_RVA: usize = rva(0x614770);
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn clear_forces() -> Result<Left, Refused> {
    unsafe { clear_records(game_addr(RECORDS_RVA) as u32, COUNT) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_forces() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
