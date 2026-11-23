//! Turns the driven wheels (`driven_wheel_turns`) on by `drive_speed_a` / `drive_speed_b` over
//! `wheel_spin_share`, times the step `step_time`, 65536 and `one_over_pi` (rounded through
//! `scratch`). These are the driven wheels' turn angles, for the drawing.
//!
//! Returns the second driven wheel's turn.

use crate::math::x87::div_mul3_to_int;
use crate::physics::car_store::Car;
use crate::physics::figures::WHEEL_SPIN_SHARE;
use crate::race::numbers_store::Numbers;

/// The exact version, for `physics::car_physics_step`: the last turn added.
pub(super) fn spin_wheels(car: &mut Car, numbers: &Numbers) -> u32 {
    let (wheel, dt, k65536, scale) = (
        WHEEL_SPIN_SHARE,
        numbers.step_time.get(),
        &numbers.k65536,
        &numbers.one_over_pi,
    );
    let mut last = 0;
    let [turn_a, turn_b] = &mut car.driven_wheel_turns;
    for (speed, turn) in [(car.drive_speed_a, turn_a), (car.drive_speed_b, turn_b)] {
        last = div_mul3_to_int(speed, wheel, dt, k65536, scale) as u32;
        *turn = turn.wrapping_add(last);
    }
    last
}
