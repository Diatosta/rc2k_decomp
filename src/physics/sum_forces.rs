//! Five sums over gravity (`gravity`), the first three also over the step `step_time`:
//! `force_sums[0]` = `drive_slip[0]`, `force_sums[1]` = `drive_slip[1]`, `force_sums[2]` =
//! `axle_slips[0]` + `axle_slips[1]`, `force_sums[3]` = the y forces of force records 6 .. 9,
//! `force_sums[4]` = those of records 0 .. 4 (the records `physics::clear_forces` clears). The
//! step's forces summed, over gravity (so in g).

use crate::math::x87::sum_over;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::wheel_work::WheelWork;
use std::cell::RefCell;

/// The exact x87 version, for `physics::car_physics_step`.
pub fn sum_forces(tables: &PhysicsTables, work: &RefCell<WheelWork>) {
    let g = tables.gravity.get();
    let dt = tables.step_time.get();
    let up = |k: usize| tables.forces[k].force[1].get();
    let (drive_slip, axle_slips) = {
        let work = work.borrow();
        (
            work.drive_slip.map(f32::from_bits),
            work.axle_slips.map(f32::from_bits),
        )
    };
    let sums = [
        sum_over(&[drive_slip[0]], &[g, dt]),
        sum_over(&[drive_slip[1]], &[g, dt]),
        sum_over(&[axle_slips[0], axle_slips[1]], &[g, dt]),
        sum_over(&[up(6), up(7), up(8), up(9)], &[g]),
        sum_over(&[up(0), up(1), up(2), up(3), up(4)], &[g]),
    ];
    work.borrow_mut().force_sums = sums.map(f32::to_bits);
}
