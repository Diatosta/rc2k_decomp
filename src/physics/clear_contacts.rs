//! `physics::clear_contacts` (0x43F3D6): `physics::clear_records` on 7 records of 0x20 bytes at `contacts` (`g_contactRecords`, the
//! step's contact forces).
//!
//! Caller: `physics::car_physics_step` (0x43B85B), every physics step.

use crate::errors::Refused;
use crate::physics::clear_records::clear_records;
use crate::physics::tables_store::PhysicsTables;
pub fn clear_contacts(tables: &PhysicsTables) -> Result<(), Refused> {
    clear_records(&tables.contacts);
    Ok(())
}
