//! Applies the 7 contact force records (`contacts`) to the work car with `physics::apply_forces`.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
const COUNT: usize = 7;

pub fn apply_contact_forces(car: &Car, tables: &PhysicsTables) -> Result<(), Refused> {
    apply_forces(car, tables, &tables.contacts[..COUNT])
}
