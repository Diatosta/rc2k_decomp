//! `physics::apply_contact_forces` (0x43F555): `physics::apply_forces` on the 7 records at `unk_614A50` (`physics::clear_contacts`').
//!
//! In:  nothing
//! Out: esi the first record, ecx the count; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B883), every physics step.
//! Self-test: a copy of the original (its records and car the test's, its call to a
//! copy of `apply_forces`) against the Rust core.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ECX, ESI, game_addr, rva};
use crate::physics::apply_forces::apply_forces;
use crate::screens::harness::Left;

const RECORDS_RVA: usize = rva(0x614A50);
const COUNT: u32 = 7;

/// The Rust version.
pub unsafe fn apply_contact_forces() -> Result<Left, Refused> {
    let first = game_addr(RECORDS_RVA) as u32;
    unsafe { apply_forces(first, COUNT) }?;
    Ok(Left::nothing().with(ESI, first).with(ECX, COUNT))
}
