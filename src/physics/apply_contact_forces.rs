//! `physics::apply_contact_forces` (0x43F555): `physics::apply_forces` on the 7 records at `unk_614A50` (`physics::clear_contacts`').
//!
//! Returns: the first record, the count
//!
//! Caller: `physics::car_physics_step` (0x43B883), every physics step.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
const COUNT: usize = 7;

pub fn apply_contact_forces(
    car: &Car,
    tables: &PhysicsTables,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    apply_forces(car, tables, &tables.contacts[..COUNT], scene)
}
