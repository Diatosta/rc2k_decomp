//! Applies a table of force records (0x20 bytes each) to the work car, one `physics::apply_force`
//! each.
//!
//! Refuses no records, or more than 0x100.

use crate::errors::Refused;
use crate::physics::apply_force::apply_force;
use crate::physics::car_store::Car;
use crate::physics::tables_store::ForceRecord;
use crate::race::numbers_store::Numbers;

/// `records` (a whole table, the wheels' 0x17 or the contacts' 7) on the work car.
pub(super) fn apply_forces(
    car: &Car,
    numbers: &Numbers,
    records: &[ForceRecord],
) -> Result<(), Refused> {
    for rec in records {
        apply_force(car, numbers, rec);
    }
    Ok(())
}
