//! Adds the drive forces: a factor (`drive_force_factor`, or its second entry when
//! the car's drive kind is set) times the sum of the four wheel loads is the scale; then
//! `drive_forces[0]` and `drive_forces[1]` gain `wheel_slips[0]` and `wheel_slips[1]` times that
//! scale times `drive_shares[2]`.

use crate::math::x87;
use crate::math::x87::mul2_plus;
use crate::physics::car_store::Car;
use crate::physics::figures::DRIVE_FORCE_FACTOR;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::wheel_work::WheelWork;
use std::cell::RefCell;

/// The forces' step.
pub fn add_drive_forces(car: &Car, tables: &PhysicsTables, work: &RefCell<WheelWork>) {
    let mut factor = DRIVE_FORCE_FACTOR[0];
    if car.drive_kind.get() != 0 {
        factor = DRIVE_FORCE_FACTOR[1];
    }
    let loads = &tables.wheel_loads;
    let scale = x87::sum4_times(loads[0], loads[1], loads[2], loads[3], factor);
    let share = tables.drive_shares[2];
    let mut work = work.borrow_mut();
    for axle in 0..2 {
        work.drive_forces[axle] = mul2_plus(
            work.wheel_slips[axle],
            scale,
            share,
            work.drive_forces[axle],
        )
        .to_bits();
    }
}
