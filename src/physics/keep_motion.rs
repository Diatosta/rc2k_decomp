//! `physics::keep_motion` (0x440B7C): `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (0x71A184) into `dword_61409C`, `dword_6140A0`, `dword_6140A4` (eax pushed around).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B87E), every physics step.

use crate::framework::{game_addr, rva};

/// From, to, three times (IDA addresses).
const VARIABLES: [usize; 6] = [0x71A17C, 0x71A180, 0x71A184, 0x61409C, 0x6140A0, 0x6140A4];

unsafe fn core(v: &[u32]) {
    for k in 0..3 {
        unsafe { (v[k + 3] as *mut u32).write_unaligned((v[k] as *const u32).read_unaligned()) };
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn keep_motion() {
    unsafe { core(&game()) }
}
