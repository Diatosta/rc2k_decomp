//! `drive::clear_start_state` (0x4376E3): six of the work copy's start values 0:
//! `dword_71A234` (the grades' mode, `physics::grade_or_reset`), `dword_71A070`,
//! `dword_71AA88`, `flt_71A650`, `flt_71A658`, `dword_71A01C`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::store_car` next)
//!
//! Caller: `sub_43703E` (0x4371BC, 0x43729B).
//! Self-test: `physics/copy_hooks_test.rs`, 4 fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, rva};

const VARIABLES: [usize; 6] = [0x71A234, 0x71A070, 0x71AA88, 0x71A650, 0x71A658, 0x71A01C];

unsafe fn core(v: &[u32]) {
    for &a in v {
        unsafe { (a as *mut u32).write_unaligned(0) };
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn clear_start_state() {
    unsafe { core(&game()) }
}
