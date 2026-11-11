//! `physics::air_drag` (0x43F2DA): for the motion's three (`local_velocity`,
//! `dword_71A180`, `g_carSpeed`): its square times `air_drag` over `k10000` into
//! `flt_614810` .. `flt_614818`, made against the motion (|it| when the motion is below 0
//! or no number, else -|it|). This is the air's drag: a force against the motion that grows with the square of the speed.
//!
//! Caller: `physics::car_physics_step` (0x43B80E), every physics step.

use crate::math::x87;
use crate::math::x87::{abs_of, compare_full, square_times_over};
use crate::physics::car_store::Car;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::session::Cell32;

/// On the variables' cells `v`.
pub fn drag_at(tables: &PhysicsTables, numbers: &Numbers, v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    let (k, d, zero) = (tables.air_drag.bits(), numbers.k10000.bits(), ZERO);
    for i in 0..3 {
        put(3 + i, square_times_over(get(i), k, d));
        let a = abs_of(get(3 + i));
        let below = compare_full(get(i), zero) & 0x100 != 0;
        put(3 + i, if below { a } else { x87::negated(a) });
    }
}

fn game<'a>(car: &'a Car, tables: &'a PhysicsTables) -> Vec<&'a dyn Cell32> {
    vec![
        &car.local_velocity[0] as &dyn Cell32,
        &car.local_velocity[1],
        &car.local_velocity[2],
        &tables.forces[5].force[0],
        &tables.forces[5].force[1],
        &tables.forces[5].force[2],
    ]
}

/// The exact version, for `physics::car_physics_step`.
pub fn air_drag(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    drag_at(tables, numbers, &game(car, tables));
}
