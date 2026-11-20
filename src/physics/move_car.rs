//! Moves the work car's place (`position`, whole numbers) by its motion (`velocity`, floats) times
//! 65536 and the step `step_time`, rounded through `scratch`: x, z, then y.
//!
//! Returns y's move.

use crate::math::x87::mul2_to_int;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;

/// The exact version, for `physics::car_physics_step` and `physics::apply_wheel_torques`: the last
/// move.
pub fn move_car(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> u32 {
    let k65536 = numbers.k65536.get();
    let dt = tables.step_time.get();
    let mut last = 0;
    // x, then z, then y.
    for k in [0, 2, 1] {
        last = mul2_to_int(car.velocity[k].get(), k65536, dt) as u32;
        car.position[k].set(car.position[k].get().wrapping_add(last));
    }
    last
}
