//! `physics::move_car` (0x43CCB0): the work car's place (`position` .. `114`, whole
//! numbers) moved by its motion (`velocity` .. `178`, floats) times 65536 and the step
//! `step_time` (`fistp` through `scratch`), x, z, then y.
//!
//! Returns: y's move
//!
//! Callers: `physics::car_physics_step` (0x43B827, 0x43B8A0), every physics step.

use crate::math::x87::mul2_to_int;
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

/// On the variables' cells `v`: the last move.
pub fn move_at(tables: &PhysicsTables, numbers: &Numbers, v: &[&dyn Cell32]) -> u32 {
    let k65536 = numbers.k65536.get();
    let dt = tables.step_time.get();
    let mut last = 0;
    for (motion, place) in [(0, 3), (2, 5), (1, 4)] {
        last = mul2_to_int(get(v, motion), k65536, dt) as u32;
        put(v, 6, last);
        put(v, place, get(v, place).wrapping_add(last));
    }
    last
}

fn game<'a>(car: &'a Car, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    vec![
        &car.velocity[0] as &dyn Cell32,
        &car.velocity[1],
        &car.velocity[2],
        &car.position[0],
        &car.position[1],
        &car.position[2],
        &scene.drawing.scene.scratch,
    ]
}

/// The exact version, for `physics::car_physics_step` and `physics::apply_wheel_torques`.
pub fn move_car(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    scene: &crate::scene::owner::Scene,
) -> u32 {
    move_at(tables, numbers, &game(car, scene))
}
