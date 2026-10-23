//! `physics::air_drag` (0x43F2DA): for the motion's three (`dword_71A17C`,
//! `dword_71A180`, `g_carSpeed`): its square times `flt_613FCC` over `flt_5C5E34` into
//! `flt_614810` .. `flt_614818`, made against the motion (|it| when the motion is below 0
//! or no number, else -|it|). By the look of it the air's drag (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B80E), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 8 fillings (signs, zeros, NaN), exact at
//! double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::{abs_of, compare_full, square_times_over};

const VARIABLES: [usize; 6] = [0x71A17C, 0x71A180, 0x71A184, 0x614810, 0x614814, 0x614818];

/// The Rust core on the variables at `v`.
pub unsafe fn drag_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let (k, d, zero) = (
            game_read::<u32>(rva(0x613FCC)),
            game_read::<u32>(rva(0x5C5E34)),
            game_read::<u32>(rva(0x5C5D90)),
        );
        for i in 0..3 {
            put(3 + i, square_times_over(get(i), k, d));
            let a = abs_of(get(3 + i));
            let below = compare_full(get(i), zero) & 0x100 != 0;
            put(3 + i, if below { a } else { x87::negated(a) });
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn air_drag() {
    unsafe { drag_at(&game()) }
}
