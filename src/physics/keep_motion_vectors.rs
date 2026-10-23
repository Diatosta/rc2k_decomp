//! `physics::keep_motion_vectors` (0x43BD41): unless `dword_614148` is set, the three
//! dwords at `dword_71A170` (the vector `physics::store_motion_length` measures) into
//! `dword_614BCC` .. `D4`, and those at `dword_71A15C` into `dword_614BDC` .. `E4` (eax
//! pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B89B), every physics step.
//! Self-test: `physics/copy_hooks_test.rs`, 8 fillings, the switch on and off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, rva};

/// The switch, six from, six to (IDA addresses).
const VARIABLES: [usize; 13] = [
    0x614148, 0x71A170, 0x71A174, 0x71A178, 0x71A15C, 0x71A160, 0x71A164, 0x614BCC, 0x614BD0,
    0x614BD4, 0x614BDC, 0x614BE0, 0x614BE4,
];

unsafe fn core(v: &[u32]) {
    unsafe {
        if (v[0] as *const u32).read_unaligned() != 0 {
            return;
        }
        for k in 0..6 {
            (v[k + 7] as *mut u32).write_unaligned((v[k + 1] as *const u32).read_unaligned());
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn keep_motion_vectors() {
    unsafe { core(&game()) }
}
