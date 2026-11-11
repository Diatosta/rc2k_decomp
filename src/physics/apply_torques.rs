//! `physics::apply_torques` (0x43F6BF): applies a table of force records (0x20 bytes each) as
//! torques to the work car (`g_car`), each by `physics::apply_torque`.
//!
//! Takes: the records
//!
//! Callers: `physics::apply_wheel_torques`, `physics::apply_contact_torques`. Refuses a
//! table of no records or of more than 0x100 (the original would loop 2^32 times).

use crate::errors::Refused;
use crate::physics::apply_torque::apply_torque;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;

/// `records` (a whole table, the wheels' 0x17 or the contacts' 7) on the work car.
pub fn apply_torques(
    tables: &PhysicsTables,
    contact: &ContactWork,
    car: &Car,
    records: &[ForceRecord],
) -> Result<(), Refused> {
    for rec in records {
        apply_torque(tables, contact, rec, car);
    }
    Ok(())
}
