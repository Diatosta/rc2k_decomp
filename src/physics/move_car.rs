//! Moves the work car's place (`position`, whole numbers) by its motion (`velocity`, floats) times
//! 65536 and the step `step_time`, rounded through `scratch`: x, z, then y.
//!
//! Returns y's move.

use crate::math::x87::mul2_to_int;
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;

/// The exact version, for `physics::car_physics_step` and `physics::apply_wheel_torques`: the last
/// move.
pub(super) fn move_car(car: &Car, numbers: &Numbers) -> u32 {
    let k65536 = numbers.k65536.get();
    let dt = numbers.step_time.get();
    let mut last = 0;
    // x, then z, then y.
    for k in [0, 2, 1] {
        last = mul2_to_int(car.velocity.at(k), k65536, dt) as u32;
        car.position
            .set_bits_at(k, car.position.bits_at(k).wrapping_add(last));
    }
    last
}
