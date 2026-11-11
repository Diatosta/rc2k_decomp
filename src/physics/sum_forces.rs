//! `physics::sum_forces` (0x43BBCB): five sums over gravity (`gravity`), the first
//! three also over the step `step_time`: `force_sums` = `drive_slip`, `flt_614C30` =
//! `flt_614254`, `flt_614C34` = `axle_slips` + `flt_61425C`, `flt_614C38` = the four at
//! 0x614834 + 0x20 k, `flt_614C3C` = the five at 0x614774 + 0x20 k (the records
//! `physics::clear_forces` clears). The step's forces summed, over gravity (so in g).
//!
//! Caller: `physics::car_physics_step` (0x43B995), every physics step.

use crate::math::x87::sum_over;
use crate::physics::tables_store::PhysicsTables;
use crate::session::Cell32;

/// On the variables' cells `v`.
pub fn sum_at(tables: &PhysicsTables, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let g: u32 = tables.gravity.bits();
    let dt: u32 = tables.step_time.bits();
    put(13, sum_over(&[get(0)], &[g, dt]));
    put(14, sum_over(&[get(1)], &[g, dt]));
    put(15, sum_over(&[get(2), get(3)], &[g, dt]));
    put(16, sum_over(&[get(4), get(5), get(6), get(7)], &[g]));
    put(
        17,
        sum_over(&[get(8), get(9), get(10), get(11), get(12)], &[g]),
    );
}

fn game(tables: &PhysicsTables) -> Vec<&dyn Cell32> {
    vec![
        &tables.drive_slip[0] as &dyn Cell32,
        &tables.drive_slip[1],
        &tables.axle_slips[0],
        &tables.axle_slips[1],
        &tables.forces[6].force[1],
        &tables.forces[7].force[1],
        &tables.forces[8].force[1],
        &tables.forces[9].force[1],
        &tables.forces[0].force[1],
        &tables.forces[1].force[1],
        &tables.forces[2].force[1],
        &tables.forces[3].force[1],
        &tables.forces[4].force[1],
        &tables.force_sums[0],
        &tables.force_sums[1],
        &tables.force_sums[2],
        &tables.force_sums[3],
        &tables.force_sums[4],
    ]
}

/// The exact x87 version, for `physics::car_physics_step`.
pub fn sum_forces(tables: &PhysicsTables) {
    sum_at(tables, &game(tables));
}
