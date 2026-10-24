//! `physics::take_split` (0x436900, race modes other than 2): unless finished (+0xA8),
//! when a checkpoint is past the splits taken (+0xAC below +0x90, unsigned):
//! `physics::split_by_mode`, its figure (+0xA0C) kept at +0x140 [split], the splits taken
//! made +0x90; at `g_splitColourCount` the car is finished: +0xA8 1, +0xA0 0, the notice
//! 0xC8 (+0x72C -1, +0x730 0).
//!
//! In:  nothing
//! Out: eax, ebx and what `split_by_mode` leaves; no flags read
//!
//! Caller: `physics::race_timers`. Refuses (before anything) a split past 0x3F, and what
//! `split_by_mode` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::splits::{CHECKS, COUNT, DONE, FIGURE, SPLIT, SPLITS, g, p};
use rc2k_core::physics::progress::{FINISH_NOTICE, finished_at, split_due};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split() -> Result<(), Refused> {
    if g(DONE) != 0 {
        return Ok(());
    }
    if !split_due(g(SPLIT), g(CHECKS))? {
        return Ok(());
    }
    split_by_mode()?;
    p(SPLITS + 4 * g(SPLIT) as usize, g(FIGURE));
    p(SPLIT, g(CHECKS));
    if finished_at(g(SPLIT), g(COUNT)) {
        p(DONE, 1);
        p(0x71A078, 0);
        p(0x71A6F4, FINISH_NOTICE);
        p(0x71A704, u32::MAX);
        p(0x71A708, 0);
    }
    Ok(())
}
