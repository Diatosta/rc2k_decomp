//! Applies the 0x17 force records (`forces`, the wheels' among them) to the work car with
//! `physics::apply_forces`.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
const COUNT: usize = 0x17;

pub(super) fn apply_wheel_forces(
    car: &mut Car,
    numbers: &Numbers,
    tables: &PhysicsTables,
) -> Result<(), Refused> {
    apply_forces(car, numbers, &tables.forces[..COUNT])
}
