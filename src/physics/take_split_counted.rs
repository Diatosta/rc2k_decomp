//! `physics::take_split_counted` (0x436833, race mode 2): for car 0 only, when a
//! checkpoint is past the splits taken (+0xAC below +0x90, unsigned):
//! `physics::keep_split_time` (inside `pusha` / `popa`), `physics::set_flag_51659c`,
//! `physics::split_by_mode`, the figure kept at +0x140 [split]; before the last split
//! the notice 0xC9 (+0x730 0x32, +0x72C 300) and the count-down +0x8C up by
//! `dword_601434[split]` x 100; +0xA4 0; the splits taken made +0x90; at
//! `g_splitColourCount` finished: +0xA8 1, the notice 0xC8 (+0x72C -1, +0x730 0).
//!
//! In:  nothing
//! Out: eax, ebx and what the calls leave; no flags read
//!
//! Caller: `physics::race_timers`.
//! Refuses (before anything) a split past 0x3F, and what the calls refuse.
//! Self-test: `physics/splits.rs` (`test_splits`), mode 2.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::physics::keep_split_time::keep_split_time;
use crate::physics::set_flag_51659c::set_flag_51659c;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::splits::{CHECKS, COUNT, DONE, FIGURE, SPLIT, SPLITS, g, p};
use rc2k_core::physics::progress::{
    FINISH_NOTICE, SPLIT_NOTICE, finished_at, split_bonus, split_due,
};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split_counted() -> Result<(), Refused> {
    if g(0x71B3F8) != 0 {
        return Ok(());
    }
    if !split_due(g(SPLIT), g(CHECKS))? {
        return Ok(());
    }
    unsafe { keep_split_time() }?;
    unsafe { set_flag_51659c() }?;
    split_by_mode()?;
    p(SPLITS + 4 * g(SPLIT) as usize, g(FIGURE));
    if g(CHECKS) < g(COUNT) {
        p(0x71A6F4, SPLIT_NOTICE);
        p(0x71A708, 0x32);
        p(0x71A704, 300);
        let bonus = split_bonus(g(0x601434 + 4 * g(SPLIT) as usize));
        p(0x71A064, g(0x71A064).wrapping_add(bonus));
    }
    p(0x71A07C, 0);
    p(SPLIT, g(CHECKS));
    if finished_at(g(SPLIT), g(COUNT)) {
        p(DONE, 1);
        p(0x71A6F4, FINISH_NOTICE);
        p(0x71A704, u32::MAX);
        p(0x71A708, 0);
    }
    Ok(())
}
