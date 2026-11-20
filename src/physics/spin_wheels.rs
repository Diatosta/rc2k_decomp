//! Turns the driven wheels (`driven_wheel_turns`) on by `drive_speed_a` / `drive_speed_b` over
//! `wheel_spin_share`, times the step `step_time`, 65536 and `one_over_pi` (rounded through
//! `scratch`). These are the driven wheels' turn angles, for the drawing.
//!
//! Returns the second driven wheel's turn.

use crate::math::x87::div_mul3_to_int;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;

/// The exact version, for `physics::car_physics_step`: the last turn added.
pub(super) fn spin_wheels(car: &Car, tables: &PhysicsTables, numbers: &Numbers) -> u32 {
    let (wheel, dt, k65536, scale) = (
        &tables.wheel_spin_share,
        &tables.step_time,
        &numbers.k65536,
        &numbers.one_over_pi,
    );
    let mut last = 0;
    for (speed, turn) in [
        (&car.drive_speed_a, &car.driven_wheel_turns[0]),
        (&car.drive_speed_b, &car.driven_wheel_turns[1]),
    ] {
        last = div_mul3_to_int(speed.get(), wheel, dt, k65536, scale) as u32;
        turn.set(turn.get().wrapping_add(last));
    }
    last
}
