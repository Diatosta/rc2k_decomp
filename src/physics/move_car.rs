//! `physics::move_car` (0x43CCB0): the work car's place (`dword_71A10C` .. `114`, whole
//! numbers) moved by its motion (`dword_71A170` .. `178`, floats) times 65536 and the step
//! `flt_61388C` (`fistp` through `dword_62400C`), x, z, then y.
//!
//! In:  nothing
//! Out: eax y's move; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B827, 0x43B8A0), every physics step.

use crate::math::x87_run::mul2_to_int;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_course::Course;
use crate::session::Cell32;

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x);
}

/// The Rust core on the variables' cells `v`: the last move.
pub fn move_at(course: &Course, v: &[&dyn Cell32]) -> u32 {
    let k65536: u32 = course.numbers.k65536.bits();
    let dt: u32 = PHYSICS.step_time.bits();
    let mut last = 0;
    for (motion, place) in [(0, 3), (2, 5), (1, 4)] {
        last = mul2_to_int(get(v, motion), k65536, dt) as u32;
        put(v, 6, last);
        put(v, place, get(v, place).wrapping_add(last));
    }
    last
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.velocity[0] as &dyn Cell32,
        &CAR.velocity[1],
        &CAR.velocity[2],
        &CAR.position[0],
        &CAR.position[1],
        &CAR.position[2],
        &crate::scene::owner::scene().drawing.scene.scratch,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::apply_wheel_torques`).
pub fn move_car(course: &Course) -> u32 {
    move_at(course, &game())
}
