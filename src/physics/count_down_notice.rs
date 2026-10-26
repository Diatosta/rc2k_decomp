//! `physics::count_down_notice` (0x440F08): while a notice is shown (`dword_71A700` set)
//! and its time `dword_71A704` is not 0, the time less 1; at 0 the notice is cleared
//! (`dword_71A700` 0). `physics::take_notice` sets them.
//!
//! In:  nothing
//! Out: eax the time before (when `dword_71A700` is set); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B919), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put_changed};
use rc2k_core::physics::notices::Notice;

/// The Rust version on (shown, time) at `v`. The registers the original leaves are read by none of its callers (eax).
// Converted: `rc2k_core::physics::notices::Notice::count_down`.
pub fn count_down_at(v: [u32; 2]) {
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

fn game() -> [u32; 2] {
    [
        CAR.notice_shown.addr() as u32,
        CAR.notice_time.addr() as u32,
    ]
}

/// The Rust version.
pub fn count_down_notice() {
    count_down_at(game())
}
