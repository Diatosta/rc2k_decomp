//! `physics::apply_torques` (0x43F6BF): ebx force records of 0x20 bytes from edi, each
//! by `physics::apply_torque` on the work car (`g_car`).
//!
//! Takes: the first record, their number
//!
//! Callers: `physics::apply_wheel_torques`, `physics::apply_contact_torques`. Refuses
//! (before anything) ebx 0 (the original would loop 2^32 times) or above 0x100.

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
