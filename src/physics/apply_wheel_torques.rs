//! Applies the same 0x17 records as torques (`physics::apply_torques`).

use crate::errors::Refused;
use crate::physics::apply_torques::apply_torques;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use std::cell::RefCell;
const COUNT: usize = 0x17;

pub(super) fn apply_wheel_torques(
    numbers: &Numbers,
    tables: &PhysicsTables,
    contact: &RefCell<ContactWork>,
    car: &Car,
) -> Result<(), Refused> {
    apply_torques(numbers, contact, car, &tables.forces[..COUNT])
}
