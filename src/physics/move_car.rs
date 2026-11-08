//! `physics::move_car` (0x43CCB0): the work car's place (`dword_71A10C` .. `114`, whole
//! numbers) moved by its motion (`dword_71A170` .. `178`, floats) times 65536 and the step
//! `flt_61388C` (`fistp` through `dword_62400C`), x, z, then y.
//!
//! Returns: y's move
//!
//! Callers: `physics::car_physics_step` (0x43B827, 0x43B8A0), every physics step.

use crate::math::x87_run::mul2_to_int;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::session::Cell32;

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// The Rust core on the variables' cells `v`: the last move.
pub fn move_at(race: &Race, v: &[&dyn Cell32]) -> u32 {
    let k65536: u32 = race.course.numbers.k65536.bits();
    let dt: u32 = race.shared.physics_tables.step_time.bits();
    let mut last = 0;
    for (motion, place) in [(0, 3), (2, 5), (1, 4)] {
        last = mul2_to_int(get(v, motion), k65536, dt) as u32;
        put(v, 6, last);
        put(v, place, get(v, place).wrapping_add(last));
    }
    last
}

fn game<'a>(shared: &'a Shared, scene: &'a crate::scene::owner::Scene) -> Vec<&'a dyn Cell32> {
    vec![
        &shared.car.velocity[0] as &dyn Cell32,
        &shared.car.velocity[1],
        &shared.car.velocity[2],
        &shared.car.position[0],
        &shared.car.position[1],
        &shared.car.position[2],
        &scene.drawing.scene.scratch,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::apply_wheel_torques`).
pub fn move_car(race: &Race, scene: &crate::scene::owner::Scene) -> u32 {
    move_at(race, &game(&race.shared, scene))
}
