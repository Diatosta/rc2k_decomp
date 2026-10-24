//! `physics::copy_car_word_8` (0x440C66): `g_car` +8 (`dword_719FE0`) into
//! `dword_6140F0`, and `dword_6140EC` 0 (in the gearbox: by the look of it the moment a
//! gear change began; a reading).
//!
//! In:  nothing
//! Out: eax the word; the rest kept; no flags read
//!
//! Caller: `sub_43C8AC`.

use crate::framework::{game_addr, rva};

/// The word, its copy, the cleared one (IDA addresses).
const VARIABLES: [usize; 3] = [0x719FE0, 0x6140F0, 0x6140EC];

unsafe fn core(v: &[u32]) {
    unsafe {
        let w = (v[0] as *const u32).read_unaligned();
        (v[1] as *mut u32).write_unaligned(w);
        (v[2] as *mut u32).write_unaligned(0);
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn copy_car_word_8() {
    unsafe { core(&game()) }
}
