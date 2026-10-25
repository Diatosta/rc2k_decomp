//! `physics::air_drag` (0x43F2DA): for the motion's three (`dword_71A17C`,
//! `dword_71A180`, `g_carSpeed`): its square times `flt_613FCC` over `flt_5C5E34` into
//! `flt_614810` .. `flt_614818`, made against the motion (|it| when the motion is below 0
//! or no number, else -|it|). By the look of it the air's drag (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B80E), every physics step.

use crate::math::x87;
use crate::math::x87_run::{abs_of, compare_full, square_times_over};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;

/// The Rust core on the variables at `v`.
pub unsafe fn drag_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let (k, d, zero) = (
            PHYSICS.air_drag.bits(),
            NUMBERS.k10000.bits(),
            NUMBERS.zero.bits(),
        );
        for i in 0..3 {
            put(3 + i, square_times_over(get(i), k, d));
            let a = abs_of(get(3 + i));
            let below = compare_full(get(i), zero) & 0x100 != 0;
            put(3 + i, if below { a } else { x87::negated(a) });
        }
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.local_velocity[0].addr() as u32,
        CAR.local_velocity[1].addr() as u32,
        CAR.local_velocity[2].addr() as u32,
        PHYSICS.forces[5].force[0].addr() as u32,
        PHYSICS.forces[5].force[1].addr() as u32,
        PHYSICS.forces[5].force[2].addr() as u32,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn air_drag() {
    unsafe { drag_at(&game()) }
}
