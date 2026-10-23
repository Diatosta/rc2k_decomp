//! `physics::clear_contacts` (0x43F3D6): `physics::clear_records` on 7 records of 0x20 bytes at `unk_614A50` (by the look of
//! it the step's contacts, a reading).
//!
//! In:  nothing
//! Out: edi past the records, ecx 0; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B85B), every physics step.
//! Self-test: a copy of the original (its records a test buffer, its call kept) against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::clear_records::clear_records;
use crate::screens::harness::Left;

const RECORDS_RVA: usize = rva(0x614A50);
const COUNT: u32 = 7;

/// The Rust version.
pub unsafe fn clear_contacts() -> Result<Left, Refused> {
    unsafe { clear_records(game_addr(RECORDS_RVA) as u32, COUNT) }
}
