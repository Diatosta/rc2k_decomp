//! Five sums over gravity (`gravity`), the first three also over the step `step_time`:
//! `force_sums[0]` = `drive_slip[0]`, `force_sums[1]` = `drive_slip[1]`, `force_sums[2]` =
//! `axle_slips[0]` + `axle_slips[1]`, `force_sums[3]` = the y forces of force records 6 .. 9,
//! `force_sums[4]` = those of records 0 .. 4 (the records `physics::clear_forces` clears). The
//! step's forces summed, over gravity (so in g).

use crate::math::x87::sum_over;
use crate::physics::tables_store::PhysicsTables;

/// The exact x87 version, for `physics::car_physics_step`.
pub fn sum_forces(tables: &PhysicsTables) {
    let g = tables.gravity.get();
    let dt = tables.step_time.get();
    let up = |k: usize| tables.forces[k].force[1].get();
    let sums = &tables.force_sums;
    sums[0].set(sum_over(&[tables.drive_slip[0].get()], &[g, dt]));
    sums[1].set(sum_over(&[tables.drive_slip[1].get()], &[g, dt]));
    sums[2].set(sum_over(
        &[tables.axle_slips[0].get(), tables.axle_slips[1].get()],
        &[g, dt],
    ));
    sums[3].set(sum_over(&[up(6), up(7), up(8), up(9)], &[g]));
    sums[4].set(sum_over(&[up(0), up(1), up(2), up(3), up(4)], &[g]));
}
