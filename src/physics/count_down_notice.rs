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

use crate::physics::cells::set_changed;
use crate::race::owner::race;
use rc2k_core::physics::notices::Notice;

/// The Rust version.
pub fn count_down_notice() {
    let shared_store = &race().shared;

    let was = Notice {
        shown: shared_store.car.notice_shown.get(),
        time: shared_store.car.notice_time.get(),
        ..Notice::default()
    };
    let mut n = was;
    n.count_down();
    set_changed(&shared_store.car.notice_time, was.time, n.time);
    set_changed(&shared_store.car.notice_shown, was.shown, n.shown);
}
