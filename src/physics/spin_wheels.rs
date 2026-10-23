//! `physics::spin_wheels` (0x43CD1A): the driven wheels' turns (`dword_71A1D0`,
//! `dword_71A1D4`) on by `g_carDriveSpeed` / `g_carDriveSpeed2` over `flt_613EBC`, times
//! the step `flt_61388C`, 65536 and `flt_5C5DC8` (`fistp` through `dword_62400C`). By the
//! look of it the wheels' angles for the drawing (a reading).
//!
//! In:  nothing
//! Out: eax the second's turn; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B90A), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 8 fillings, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87_run::div_mul3_to_int;

/// Speed 1, speed 2, turn 1, turn 2, scratch.
const VARIABLES: [usize; 5] = [0x71A190, 0x71A194, 0x71A1D0, 0x71A1D4, 0x62400C];

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust core on the variables at `v`: the last turn added.
pub unsafe fn spin_at(v: &[u32]) -> u32 {
    unsafe {
        let k = |x: usize| game_read::<u32>(rva(x));
        let (wheel, dt, k65536, scale) = (k(0x613EBC), k(0x61388C), k(0x5C5D98), k(0x5C5DC8));
        let mut last = 0;
        for (speed, turn) in [(0, 2), (1, 3)] {
            last = div_mul3_to_int(get(v, speed), wheel, dt, k65536, scale) as u32;
            put(v, 4, last);
            put(v, turn, get(v, turn).wrapping_add(last));
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

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn spin_wheels() -> u32 {
    unsafe { spin_at(&game()) }
}
