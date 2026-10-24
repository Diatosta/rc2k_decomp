//! `physics::count_down_notice` (0x440F08): while a notice is shown (`dword_71A700` set)
//! and its time `dword_71A704` is not 0, the time less 1; at 0 the notice is cleared
//! (`dword_71A700` 0). `physics::take_notice` sets them.
//!
//! In:  nothing
//! Out: eax the time before (when `dword_71A700` is set); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B919), every physics step.
//! Self-test: a copy of the original on test variables, shown or not, times 0, 1, 2,
//! large.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::framework::{game_addr, rva};
use crate::physics::cells::{get, put_changed};
use rc2k_core::physics::notices::Notice;

/// The Rust version on (shown, time) at `v`. The registers the original leaves are read by none of its callers (eax).
// Converted: `rc2k_core::physics::notices::Notice::count_down`.
pub unsafe fn count_down_at(v: [u32; 2]) {
    unsafe {
        let was = Notice {
            shown: get(v[0]),
            time: get(v[1]),
            ..Notice::default()
        };
        let mut n = was;
        n.count_down();
        put_changed(v[1], was.time, n.time);
        put_changed(v[0], was.shown, n.shown);
    }
}

fn game() -> [u32; 2] {
    [0x71A700usize, 0x71A704].map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version.
pub unsafe fn count_down_notice() {
    unsafe { count_down_at(game()) }
}
