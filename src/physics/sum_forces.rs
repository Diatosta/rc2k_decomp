//! `physics::sum_forces` (0x43BBCB): five sums over gravity (`flt_613FC8`), the first
//! three also over the step `flt_61388C`: `flt_614C2C` = `flt_614250`, `flt_614C30` =
//! `flt_614254`, `flt_614C34` = `flt_614258` + `flt_61425C`, `flt_614C38` = the four at
//! 0x614834 + 0x20 k, `flt_614C3C` = the five at 0x614774 + 0x20 k (the records
//! `physics::clear_forces` clears). By the look of it the step's forces in g (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B995), every physics step.

use crate::math::x87_run::sum_over;
use crate::physics::tables_store::PHYSICS;

/// The Rust core on the variables at `v`.
pub unsafe fn sum_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (crate::pointers::host(v[k]) as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (crate::pointers::host(v[k]) as *mut u32).write_unaligned(x);
        let g: u32 = PHYSICS.gravity.bits();
        let dt: u32 = PHYSICS.step_time.bits();
        put(13, sum_over(&[get(0)], &[g, dt]));
        put(14, sum_over(&[get(1)], &[g, dt]));
        put(15, sum_over(&[get(2), get(3)], &[g, dt]));
        put(16, sum_over(&[get(4), get(5), get(6), get(7)], &[g]));
        put(
            17,
            sum_over(&[get(8), get(9), get(10), get(11), get(12)], &[g]),
        );
    }
}

fn game() -> Vec<u32> {
    vec![
        PHYSICS.drive_slip[0].addr(),
        PHYSICS.drive_slip[1].addr(),
        PHYSICS.axle_slips[0].addr(),
        PHYSICS.axle_slips[1].addr(),
        PHYSICS.forces[6].force[1].addr(),
        PHYSICS.forces[7].force[1].addr(),
        PHYSICS.forces[8].force[1].addr(),
        PHYSICS.forces[9].force[1].addr(),
        PHYSICS.forces[0].force[1].addr(),
        PHYSICS.forces[1].force[1].addr(),
        PHYSICS.forces[2].force[1].addr(),
        PHYSICS.forces[3].force[1].addr(),
        PHYSICS.forces[4].force[1].addr(),
        PHYSICS.force_sums[0].addr(),
        PHYSICS.force_sums[1].addr(),
        PHYSICS.force_sums[2].addr(),
        PHYSICS.force_sums[3].addr(),
        PHYSICS.force_sums[4].addr(),
    ]
}

/// The Rust version (the exact x87 one, for `physics::car_physics_step`).
pub unsafe fn sum_forces() {
    unsafe { sum_at(&game()) }
}
