//! Applies the 7 contact records as torques (`physics::apply_torques`), only when `contact_step`
//! is 0.

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
