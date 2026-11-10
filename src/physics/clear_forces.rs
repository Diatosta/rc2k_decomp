//! `physics::clear_forces` (0x43F395): `physics::clear_records` on 0x17 records of 0x20 bytes at `unk_614770` (`g_forceRecords`, the
//! step's forces).
//!
//! Caller: `physics::car_physics_step` (0x43B71E), every physics step.

use crate::errors::Refused;
use crate::physics::clear_records::clear_records;
use crate::race::owner_shared::Shared;
const COUNT: u32 = 0x17;

pub fn clear_forces(shared: &Shared) -> Result<(), Refused> {
    clear_records(&shared.physics_tables.forces, COUNT)
}
