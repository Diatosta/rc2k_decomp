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
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust core on the variables at `v`: the last move.
pub unsafe fn move_at(v: &[u32]) -> u32 {
    unsafe {
        let k65536: u32 = NUMBERS.k65536.bits();
        let dt: u32 = PHYSICS.step_time.bits();
        let mut last = 0;
        for (motion, place) in [(0, 3), (2, 5), (1, 4)] {
            last = mul2_to_int(get(v, motion), k65536, dt) as u32;
            put(v, 6, last);
            put(v, place, get(v, place).wrapping_add(last));
        }
        last
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.velocity[0].addr() as u32,
        CAR.velocity[1].addr() as u32,
        CAR.velocity[2].addr() as u32,
        CAR.position[0].addr() as u32,
        CAR.position[1].addr() as u32,
        CAR.position[2].addr() as u32,
        DRAW.scratch.addr() as u32,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::apply_wheel_torques`).
pub unsafe fn move_car() -> u32 {
    unsafe { move_at(&game()) }
}
