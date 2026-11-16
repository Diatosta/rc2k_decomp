//! Applies the 0x17 force records (`forces`, the wheels' among them) to the work car with
//! `physics::apply_forces`.

use crate::errors::Refused;
use crate::physics::apply_forces::apply_forces;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
const COUNT: usize = 0x17;

pub fn apply_wheel_forces(
    car: &Car,
    tables: &PhysicsTables,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    apply_forces(car, tables, &tables.forces[..COUNT], scene)
}
