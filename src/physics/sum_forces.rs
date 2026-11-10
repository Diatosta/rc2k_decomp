//! `physics::sum_forces` (0x43BBCB): five sums over gravity (`flt_613FC8`), the first
//! three also over the step `flt_61388C`: `flt_614C2C` = `flt_614250`, `flt_614C30` =
//! `flt_614254`, `flt_614C34` = `flt_614258` + `flt_61425C`, `flt_614C38` = the four at
//! 0x614834 + 0x20 k, `flt_614C3C` = the five at 0x614774 + 0x20 k (the records
//! `physics::clear_forces` clears). The step's forces summed, over gravity (so in g).
//!
//! Caller: `physics::car_physics_step` (0x43B995), every physics step.

use crate::math::x87_run::sum_over;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;

/// On the variables' cells `v`.
pub fn sum_at(race: &Race, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let g: u32 = race.shared.physics_tables.gravity.bits();
    let dt: u32 = race.shared.physics_tables.step_time.bits();
    put(13, sum_over(&[get(0)], &[g, dt]));
    put(14, sum_over(&[get(1)], &[g, dt]));
    put(15, sum_over(&[get(2), get(3)], &[g, dt]));
    put(16, sum_over(&[get(4), get(5), get(6), get(7)], &[g]));
    put(
        17,
        sum_over(&[get(8), get(9), get(10), get(11), get(12)], &[g]),
    );
}

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.physics_tables.drive_slip[0] as &dyn Cell32,
        &shared.physics_tables.drive_slip[1],
        &shared.physics_tables.axle_slips[0],
        &shared.physics_tables.axle_slips[1],
        &shared.physics_tables.forces[6].force[1],
        &shared.physics_tables.forces[7].force[1],
        &shared.physics_tables.forces[8].force[1],
        &shared.physics_tables.forces[9].force[1],
        &shared.physics_tables.forces[0].force[1],
        &shared.physics_tables.forces[1].force[1],
        &shared.physics_tables.forces[2].force[1],
        &shared.physics_tables.forces[3].force[1],
        &shared.physics_tables.forces[4].force[1],
        &shared.physics_tables.force_sums[0],
        &shared.physics_tables.force_sums[1],
        &shared.physics_tables.force_sums[2],
        &shared.physics_tables.force_sums[3],
        &shared.physics_tables.force_sums[4],
    ]
}

/// The exact x87 version, for `physics::car_physics_step`.
pub fn sum_forces(race: &Race) {
    sum_at(race, &game(&race.shared));
}
