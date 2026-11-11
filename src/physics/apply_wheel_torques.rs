//! `physics::apply_wheel_torques` (0x43F694): `physics::apply_torques` on the 0x17
//! records at `forces` (the ones `physics::apply_wheel_forces` pushes with).
//!
//! Returns: the count, the first record
//!
//! Caller: `physics::car_physics_step` (0x43B822, after `physics::apply_wheel_forces`, before
//! `physics::move_car`), every physics step.

use crate::errors::Refused;
use crate::physics::apply_torques::apply_torques;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::PhysicsTables;
const COUNT: usize = 0x17;

pub fn apply_wheel_torques(
    tables: &PhysicsTables,
    contact: &ContactWork,
    car: &Car,
) -> Result<(), Refused> {
    apply_torques(tables, contact, car, &tables.forces[..COUNT])
}
