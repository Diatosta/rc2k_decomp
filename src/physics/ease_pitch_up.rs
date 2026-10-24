//! `physics::ease_pitch_up` (0x438CBA): half the way from `g_car` +0x6B8 (`unk_71A690`)
//! up to `dword_608C20` (unsigned halving, `shr`), kept in `dword_608C24` and added to
//! +0x6B8. `physics::ease_pitch_down` (0x438CD3) is its twin towards `dword_608C1C`.
//!
//! In:  nothing
//! Out: eax the step; the rest kept; no flags read (the caller compares `dword_608C24`)
//!
//! Caller: `sub_43907F` (0x4392CD).

use crate::framework::{game_addr, rva};
use crate::physics::cells::{get, put};
use rc2k_core::physics::easing::ease_up;

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The variables (IDA addresses).
const VARIABLES: [usize; 3] = [0x608C20, 0x71A690, 0x608C24];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::easing::ease_up`.
unsafe fn core(v: &[u32]) {
    unsafe {
        let mut value = get(v[1]);
        let step = ease_up(&mut value, get(v[0]));
        put(v[2], step);
        put(v[1], value);
    }
}

/// The Rust version.
pub unsafe fn ease_pitch_up() {
    unsafe { core(&game()) }
}
