//! Applies a table of force records (0x20 bytes each) as torques to the work car, one
//! `physics::apply_torque` each.
//!
//! Refuses no records, or more than 0x100.

use crate::errors::Refused;
use crate::physics::apply_torque::apply_torque;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::ForceRecord;
use crate::race::numbers_store::Numbers;
use std::cell::RefCell;

/// `records` (a whole table, the wheels' 0x17 or the contacts' 7) on the work car.
pub(super) fn apply_torques(
    numbers: &Numbers,
    contact: &RefCell<ContactWork>,
    car: &mut Car,
    records: &[ForceRecord],
) -> Result<(), Refused> {
    for rec in records {
        apply_torque(numbers, contact, rec, car);
    }
    Ok(())
}
