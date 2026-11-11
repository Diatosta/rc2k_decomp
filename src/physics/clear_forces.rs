//! `physics::clear_forces` (0x43F395): `physics::clear_records` on 0x17 records of 0x20 bytes at `forces` (`g_forceRecords`, the
//! step's forces).
//!
//! Caller: `physics::car_physics_step` (0x43B71E), every physics step.

use crate::errors::Refused;
use crate::physics::clear_records::clear_records;
use crate::physics::tables_store::PhysicsTables;
const COUNT: u32 = 0x17;

pub fn clear_forces(tables: &PhysicsTables) -> Result<(), Refused> {
    clear_records(&tables.forces, COUNT)
}
