//! `physics::apply_contact_torques` (0x43F6A5): when `dword_614148` is 0,
//! `physics::apply_torques` on the 7 records at `unk_614A50` (the ones
//! `physics::apply_contact_forces` pushes with); else nothing.
//!
//! Out: `dword_614148` 0: ebx the count, edi the first record; else nothing changed;
//!      the rest kept; no flags read (the caller's next instruction is a `cmp`)
//!
//! Caller: `sub_43B635` (0x43B888, after `physics::apply_contact_forces`), every physics
//! step.

use crate::errors::Refused;
use crate::physics::apply_torques::apply_torques;
use crate::physics::tables_store::ForceRecord;
use crate::race::owner::Race;
const COUNT: usize = 7;

/// The Rust core, the switch given, `run` doing the records (`apply_torques`).
pub fn apply_contact_torques_with<'a>(
    switch: u32,
    records: &'a [ForceRecord],
    run: impl FnOnce(&'a [ForceRecord]) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if switch != 0 {
        return Ok(());
    }
    run(records)
}

/// The Rust version.
pub fn apply_contact_torques(race: &Race) -> Result<(), Refused> {
    apply_contact_torques_with(
        race.shared.physics_tables.contact_step.get(),
        &race.shared.physics_tables.contacts[..COUNT],
        |records| apply_torques(race, records),
    )
}
