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

use crate::errors::Refused;
use crate::physics::apply_torques::apply_torques;
use crate::physics::tables_store::PHYSICS;
const COUNT: u32 = 7;

/// The Rust core, the switch given, `run` doing the records (`apply_torques`).
pub fn apply_contact_torques_with(
    switch: u32,
    first: u32,
    run: impl FnOnce(u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if switch != 0 {
        return Ok(());
    }
    run(first, COUNT)
}

/// The Rust version.
pub unsafe fn apply_contact_torques() -> Result<(), Refused> {
    apply_contact_torques_with(
        PHYSICS.contact_step.get(),
        crate::pointers::host_number(PHYSICS.contacts.as_ptr()) as u32,
        |first, count| unsafe { apply_torques(first, count) },
    )
}
