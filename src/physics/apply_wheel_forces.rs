//! `physics::apply_wheel_forces` (0x43F544): `physics::apply_forces` on the 0x17 records at `unk_614770` (`physics::clear_forces`', the wheels' among them).
//!
//! In:  nothing
//! Out: esi the first record, ecx the count; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B81D), every physics step.
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

const RECORDS_RVA: usize = rva(0x614770);
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn apply_wheel_forces() -> Result<Left, Refused> {
    let first = game_addr(RECORDS_RVA) as u32;
    unsafe { apply_forces(first, COUNT) }?;
    Ok(Left::nothing().with(ESI, first).with(ECX, COUNT))
}
