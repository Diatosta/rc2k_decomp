//! `physics::apply_wheel_torques` (0x43F694): `physics::apply_torques` on the 0x17
//! records at `unk_614770` (the ones `physics::apply_wheel_forces` pushes with).
//!
//! In:  nothing
//! Out: ebx the count, edi the first record; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B822, after `physics::apply_wheel_forces`, before
//! `physics::move_car`), every physics step.
//! Self-test: a copy of the original (its records and car the test's, its call to a
//! copy of `apply_torques`) against the Rust core.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, EDI, game_addr, rva};
use crate::physics::apply_torques::apply_torques;
use crate::screens::harness::Left;

const RECORDS_RVA: usize = rva(0x614770);
const COUNT: u32 = 0x17;

/// The Rust version.
pub unsafe fn apply_wheel_torques() -> Result<Left, Refused> {
    let first = game_addr(RECORDS_RVA) as u32;
    unsafe { apply_torques(first, COUNT) }?;
    Ok(Left::nothing().with(EDI, first).with(EBX, COUNT))
}
