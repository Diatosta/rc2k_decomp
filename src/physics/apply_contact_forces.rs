//! Applies the 7 contact force records (`contacts`) to the work car with `physics::apply_forces`.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
const COUNT: usize = 7;

pub(super) fn apply_contact_forces(
    car: &Car,
    numbers: &Numbers,
    tables: &PhysicsTables,
) -> Result<(), Refused> {
    apply_forces(car, numbers, &tables.contacts[..COUNT])
}
