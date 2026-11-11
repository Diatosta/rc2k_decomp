//! `physics::apply_forces` (0x43F566): ecx force records of 0x20 bytes from esi, each
//! by `physics::apply_force` on the work car (`g_car`).
//!
//! Takes: the first record, their number
//!
//! Callers: `physics::apply_wheel_forces`, `physics::apply_contact_forces`. Refuses (before
//! anything) ecx 0 (the original would loop 2^32 times) or above 0x100.

use crate::errors::Refused;
use crate::physics::apply_force::apply_force;
use crate::physics::car_store::Car;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;

/// `records` (a whole table, the wheels' 0x17 or the contacts' 7) on the work car.
pub fn apply_forces(
    car: &Car,
    tables: &PhysicsTables,
    records: &[ForceRecord],
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    for rec in records {
        apply_force(car, tables, rec, scene);
    }
    Ok(())
}
