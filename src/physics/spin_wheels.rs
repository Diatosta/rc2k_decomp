//! `physics::spin_wheels` (0x43CD1A): the driven wheels' turns (`dword_71A1D0`,
//! `dword_71A1D4`) on by `g_carDriveSpeed` / `g_carDriveSpeed2` over `flt_613EBC`, times
//! the step `flt_61388C`, 65536 and `flt_5C5DC8` (`fistp` through `dword_62400C`). By the
//! look of it the wheels' angles for the drawing (a reading).
//!
//! In:  nothing
//! Out: eax the second's turn; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B90A), every physics step.

use crate::framework::{game_read, rva};
use crate::math::x87_run::div_mul3_to_int;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::session::Cell32;

fn get(v: &[&dyn Cell32], k: usize) -> u32 {
    v[k].bits()
}

fn put(v: &[&dyn Cell32], k: usize, x: u32) {
    v[k].set_bits(x)
}

/// The Rust core on the variables' cells `v`: the last turn added.
pub fn spin_at(v: &[&dyn Cell32]) -> u32 {
    let _k = |x: usize| game_read::<u32>(rva(x));
    let (wheel, dt, k65536, scale) = (
        PHYSICS.wheel_spin_share.bits(),
        PHYSICS.step_time.bits(),
        NUMBERS.k65536.bits(),
        NUMBERS.one_over_pi.bits(),
    );
    let mut last = 0;
    for (speed, turn) in [(0, 2), (1, 3)] {
        last = div_mul3_to_int(get(v, speed), wheel, dt, k65536, scale) as u32;
        put(v, 4, last);
        put(v, turn, get(v, turn).wrapping_add(last));
    }
    last
}

fn game() -> Vec<&'static dyn Cell32> {
    vec![
        &CAR.drive_speed_a as &dyn Cell32,
        &CAR.drive_speed_b,
        &CAR.driven_wheel_turns[0],
        &CAR.driven_wheel_turns[1],
        &DRAW.scratch,
    ]
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub fn spin_wheels() -> u32 {
    spin_at(&game())
}
