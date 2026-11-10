//! `physics::clear_contacts` (0x43F3D6): `physics::clear_records` on 7 records of 0x20 bytes at `unk_614A50` (by the look of
//! it the step's contacts, a reading).
//!
//! Caller: `physics::car_physics_step` (0x43B85B), every physics step.

use crate::errors::Refused;
use crate::physics::clear_records::clear_records;
use crate::race::owner_shared::Shared;
const COUNT: u32 = 7;

pub fn clear_contacts(shared: &Shared) -> Result<(), Refused> {
    clear_records(&shared.physics_tables.contacts, COUNT)
}
