//! `physics::apply_contact_torques` (0x43F6A5): when `dword_614148` is 0,
//! `physics::apply_torques` on the 7 records at `unk_614A50` (the ones
//! `physics::apply_contact_forces` pushes with); else nothing.
//!
//! In:  nothing
//! Out: `dword_614148` 0: ebx the count, edi the first record; else nothing changed;
//!      the rest kept; no flags read (the caller's next instruction is a `cmp`)
//!
//! Caller: `sub_43B635` (0x43B888, after `physics::apply_contact_forces`), every physics
//! step.
//! Self-test: a copy of the original (its switch, records and car the test's, its call
//! to a copy of `apply_torques`) against the Rust core, the switch 0 and 1.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, EDI, game_addr, game_read, rva};
use crate::physics::apply_torques::apply_torques;
use crate::screens::harness::Left;

const SWITCH_RVA: usize = rva(0x614148); // dword_614148
const RECORDS_RVA: usize = rva(0x614A50);
const COUNT: u32 = 7;

/// The Rust core, the switch given, `run` doing the records (`apply_torques`).
pub fn apply_contact_torques_with(
    switch: u32,
    first: u32,
    run: impl FnOnce(u32, u32) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    if switch != 0 {
        return Ok(Left::nothing());
    }
    run(first, COUNT)?;
    Ok(Left::nothing().with(EDI, first).with(EBX, COUNT))
}

/// The Rust version.
pub unsafe fn apply_contact_torques() -> Result<Left, Refused> {
    apply_contact_torques_with(
        unsafe { game_read::<u32>(SWITCH_RVA) },
        game_addr(RECORDS_RVA) as u32,
        |first, count| unsafe { apply_torques(first, count) },
    )
}
