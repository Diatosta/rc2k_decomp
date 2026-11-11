//! `physics::hold_spin_on_ground` (0x43BAA4): with all four wheels down (the four dwords
//! from `wheel_squeeze` not 0), the spin's x and z (`spin`, `Car.spin[2]`) held
//! to -`ground_spin_limit` .. `ground_spin_limit` (by negating that limit in place, twice for each, so
//! it comes back), and `ground_spin` = whether either now has the limit's size (C3: also
//! when no number). A reading: no tumbling on the ground.
//!
//! Caller: `physics::car_physics_step` (0x43B986), every physics step.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::session::Cell32;
use rc2k_core::physics::motion::hold_spin;

/// The converted version (`rc2k_core::physics::motion`) on the variables' cells `v`.
// Converted: `rc2k_core::physics::motion::hold_spin`.
pub fn hold_at(v: &[&dyn Cell32]) {
    let get = |k: usize| v[k].bits();
    let put = |k: usize, x: u32| v[k].set_bits(x);
    if (0..4).any(|k| get(k) == 0) {
        return;
    }
    let f = |k: usize| f32::from_bits(get(k));
    let (mut limit, mut x, mut z) = (f(4), f(5), f(6));
    let flag = hold_spin(&mut limit, &mut x, &mut z);
    put(4, limit.to_bits());
    put(5, x.to_bits());
    put(6, z.to_bits());
    put(7, flag);
}

fn game<'a>(car: &'a Car, tables: &'a PhysicsTables) -> Vec<&'a dyn Cell32> {
    vec![
        &car.wheel_squeeze[0] as &dyn Cell32,
        &car.wheel_squeeze[1],
        &car.wheel_squeeze[2],
        &car.wheel_squeeze[3],
        &tables.ground_spin_limit,
        &car.spin[0],
        &car.spin[2],
        &tables.ground_spin,
    ]
}

/// Compares and copies of floats only: the same at any precision.
pub fn hold_spin_on_ground(car: &Car, tables: &PhysicsTables) {
    hold_at(&game(car, tables));
}
