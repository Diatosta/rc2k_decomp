//! `physics::spin_wheels` (0x43CD1A): the driven wheels' turns (`driven_wheel_turns`,
//! `Car.driven_wheel_turns[1]`) on by `g_carDriveSpeed` / `g_carDriveSpeed2` over `wheel_spin_share`, times
//! the step `step_time`, 65536 and `one_over_pi` (`fistp` through `scratch`). These are the
//! driven wheels' turn angles (`driven_wheel_turns`, for the drawing).
//!
//! Returns: the second's turn
//!
//! Caller: `physics::car_physics_step` (0x43B90A), every physics step.

use crate::math::x87::div_mul3_to_int;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::session::Cell32;

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// On the variables' cells `v`: the last turn added.
pub fn spin_at(tables: &PhysicsTables, numbers: &Numbers, v: &[&dyn Cell32]) -> u32 {
    let (wheel, dt, k65536, scale) = (
        tables.wheel_spin_share.bits(),
        tables.step_time.bits(),
        numbers.k65536.bits(),
        numbers.one_over_pi.bits(),
    );
    let mut last = 0;
    for (speed, turn) in [(0, 2), (1, 3)] {
        last = div_mul3_to_int(get(v, speed), wheel, dt, k65536, scale) as u32;
        put(v, 4, last);
        put(v, turn, get(v, turn).wrapping_add(last));
    }
    last
}

fn game<'a>(car: &'a Car, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    vec![
        &car.drive_speed_a as &dyn Cell32,
        &car.drive_speed_b,
        &car.driven_wheel_turns[0],
        &car.driven_wheel_turns[1],
        &scene.drawing.scene.scratch,
    ]
}

/// The exact version, for `physics::car_physics_step`.
pub fn spin_wheels(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    spin_at(tables, numbers, &game(car, scene))
}
