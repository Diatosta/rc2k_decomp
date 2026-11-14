//! Clears the 0x17 force records of 0x20 bytes (`forces`, the step's forces) with
//! `physics::clear_records`.
//!
//! Caller: `physics::car_physics_step`, every physics step.

use crate::errors::Refused;
use crate::physics::clear_records::clear_records;
use crate::physics::tables_store::PhysicsTables;
pub fn clear_forces(tables: &PhysicsTables) -> Result<(), Refused> {
    clear_records(&tables.forces);
    Ok(())
}
