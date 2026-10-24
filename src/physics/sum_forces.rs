//! `physics::sum_forces` (0x43BBCB): five sums over gravity (`flt_613FC8`), the first
//! three also over the step `flt_61388C`: `flt_614C2C` = `flt_614250`, `flt_614C30` =
//! `flt_614254`, `flt_614C34` = `flt_614258` + `flt_61425C`, `flt_614C38` = the four at
//! 0x614834 + 0x20 k, `flt_614C3C` = the five at 0x614774 + 0x20 k (the records
//! `physics::clear_forces` clears). By the look of it the step's forces in g (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B995), every physics step.

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87_run::sum_over;

/// The thirteen read, the five written.
const VARIABLES: [usize; 18] = [
    0x614250, 0x614254, 0x614258, 0x61425C, 0x614834, 0x614854, 0x614874, 0x614894, 0x614774,
    0x614794, 0x6147B4, 0x6147D4, 0x6147F4, 0x614C2C, 0x614C30, 0x614C34, 0x614C38, 0x614C3C,
];

/// The Rust core on the variables at `v`.
pub unsafe fn sum_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let g: u32 = game_read(rva(0x613FC8));
        let dt: u32 = game_read(rva(0x61388C));
        put(13, sum_over(&[get(0)], &[g, dt]));
        put(14, sum_over(&[get(1)], &[g, dt]));
        put(15, sum_over(&[get(2), get(3)], &[g, dt]));
        put(16, sum_over(&[get(4), get(5), get(6), get(7)], &[g]));
        put(
            17,
            sum_over(&[get(8), get(9), get(10), get(11), get(12)], &[g]),
        );
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact x87 one, for `physics::car_physics_step`).
pub unsafe fn sum_forces() {
    unsafe { sum_at(&game()) }
}
