//! `physics::move_car` (0x43CCB0): the work car's place (`dword_71A10C` .. `114`, whole
//! numbers) moved by its motion (`dword_71A170` .. `178`, floats) times 65536 and the step
//! `flt_61388C` (`fistp` through `dword_62400C`), x, z, then y.
//!
//! In:  nothing
//! Out: eax y's move; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B827, 0x43B8A0), every physics step.

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87_run::mul2_to_int;

/// The motion's three, the place's three, the scratch.
const VARIABLES: [usize; 7] = [
    0x71A170, 0x71A174, 0x71A178, 0x71A10C, 0x71A110, 0x71A114, 0x62400C,
];

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust core on the variables at `v`: the last move.
pub unsafe fn move_at(v: &[u32]) -> u32 {
    unsafe {
        let k65536: u32 = game_read(rva(0x5C5D98));
        let dt: u32 = game_read(rva(0x61388C));
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
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::apply_wheel_torques`).
pub unsafe fn move_car() -> u32 {
    unsafe { move_at(&game()) }
}
