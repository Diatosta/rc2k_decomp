//! `physics::blend_wheel_load` (0x440F70): unless `flt_71A208` is below `flt_614C64`
//! (C0): a slot by the signs of 0x71A1FC, 0x71A200, 0x71A204 (4 + 2 + 1 for each below 0)
//! and `flt_71A03C[slot]` moved towards 1 by `flt_71A208` / `flt_5C5D60` held to 0 .. 1
//! (through `dword_62400C`): a + (1 - a) share. What the eight are is a reading at best.
//!
//! In:  nothing
//! Out: edx the slot (when reached); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B99A), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 16 cases, exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::{compare_full, toward_one};

/// The figure, its three signs, the scratch, then the eight slots.
const VARIABLES: [usize; 13] = [
    0x71A208, 0x71A1FC, 0x71A200, 0x71A204, 0x62400C, 0x71A03C, 0x71A040, 0x71A044, 0x71A048,
    0x71A04C, 0x71A050, 0x71A054, 0x71A058,
];

/// The Rust core on the variables at `v`; `r` as the original leaves it.
pub unsafe fn blend_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
        if less(get(0), game_read(rva(0x614C64))) {
            return;
        }
        let zero: u32 = game_read(rva(0x5C5D90));
        let one: u32 = game_read(rva(0x5C5D38));
        r.edx = 0;
        for (k, add) in [(1, 4), (2, 2), (3, 1)] {
            if less(get(k), zero) {
                r.edx += add;
            }
        }
        put(4, x87::div(get(0), game_read(rva(0x5C5D60))));
        if less(get(4), zero) {
            put(4, x87::loaded(zero));
        }
        if !less(get(4), one) {
            put(4, x87::loaded(one));
        }
        let slot = 5 + r.edx as usize;
        put(slot, toward_one(get(slot), get(4)));
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step`); `r` as the
/// original leaves it.
pub unsafe fn blend_wheel_load(r: &mut Regs) {
    unsafe { blend_at(&game(), r) }
}
