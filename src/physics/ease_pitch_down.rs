//! `physics::ease_pitch_down` (0x438CD3): half the way from `g_car` +0x6B8
//! (`unk_71A690`) down to `dword_608C1C` (unsigned halving, `shr`), kept in
//! `dword_608C24` and taken from +0x6B8. The twin of `physics::ease_pitch_up`.
//!
//! In:  nothing
//! Out: eax the step; the rest kept; no flags read
//!
//! Caller: `sub_43907F` (0x4392F3).

use crate::framework::{game_addr, rva};
use crate::physics::cells::{get, put};
use rc2k_core::physics::easing::ease_down;

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The variables (IDA addresses).
const VARIABLES: [usize; 3] = [0x71A690, 0x608C1C, 0x608C24];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::easing::ease_down`.
unsafe fn core(v: &[u32]) {
    unsafe {
        let mut value = get(v[0]);
        let step = ease_down(&mut value, get(v[1]));
        put(v[2], step);
        put(v[0], value);
    }
}

/// The Rust version.
pub unsafe fn ease_pitch_down() {
    unsafe { core(&game()) }
}
