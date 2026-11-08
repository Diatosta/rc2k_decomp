//! `physics::air_drag` (0x43F2DA): for the motion's three (`dword_71A17C`,
//! `dword_71A180`, `g_carSpeed`): its square times `flt_613FCC` over `flt_5C5E34` into
//! `flt_614810` .. `flt_614818`, made against the motion (|it| when the motion is below 0
//! or no number, else -|it|). By the look of it the air's drag (a reading).
//!
//! Caller: `physics::car_physics_step` (0x43B80E), every physics step.

use crate::math::x87;
use crate::math::x87_run::{abs_of, compare_full, square_times_over};
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;

/// The Rust core on the variables' cells `v`.
pub fn drag_at(race: &Race, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let (k, d, zero) = (
        race.shared.physics_tables.air_drag.bits(),
        race.course.numbers.k10000.bits(),
        race.course.numbers.zero.bits(),
    );
    for i in 0..3 {
        put(3 + i, square_times_over(get(i), k, d));
        let a = abs_of(get(3 + i));
        let below = compare_full(get(i), zero) & 0x100 != 0;
        put(3 + i, if below { a } else { x87::negated(a) });
    }
}

fn game(shared: &Shared) -> Vec<&dyn Cell32> {
    vec![
        &shared.car.local_velocity[0] as &dyn Cell32,
        &shared.car.local_velocity[1],
        &shared.car.local_velocity[2],
        &shared.physics_tables.forces[5].force[0],
        &shared.physics_tables.forces[5].force[1],
        &shared.physics_tables.forces[5].force[2],
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub fn air_drag(race: &Race) {
    drag_at(race, &game(&race.shared));
}
