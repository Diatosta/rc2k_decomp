//! `physics::apply_contact_torques` (0x43F6A5): when `dword_614148` is 0,
//! `physics::apply_torques` on the 7 records at `unk_614A50` (the ones
//! `physics::apply_contact_forces` pushes with); else nothing.
//!
//! Caller: `physics::car_physics_step` (0x43B888, after `physics::apply_contact_forces`), every
//! physics step.

use crate::errors::Refused;
use crate::physics::apply_torques::apply_torques;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;
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

pub fn apply_contact_torques(
    tables: &PhysicsTables,
    contact: &ContactWork,
    car: &Car,
) -> Result<(), Refused> {
    apply_contact_torques_with(
        tables.contact_step.get(),
        &tables.contacts[..COUNT],
        |records| apply_torques(tables, contact, car, records),
    )
}
