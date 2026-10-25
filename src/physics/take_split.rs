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
use crate::physics::car_store::CAR;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::split_tables_store::SPLIT_TABLES;
use crate::physics::splits::put_split_time;
use rc2k_core::physics::progress::{FINISH_NOTICE, finished_at, split_due};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split() -> Result<(), Refused> {
    if CAR.finish_flag.get() != 0 {
        return Ok(());
    }
    if !split_due(CAR.splits_recorded.get(), CAR.splits_passed.get())? {
        return Ok(());
    }
    split_by_mode()?;
    put_split_time(CAR.splits_recorded.get() as usize, CAR.split_gaps[0].get());
    CAR.splits_recorded.set(CAR.splits_passed.get());
    if finished_at(
        CAR.splits_recorded.get(),
        SPLIT_TABLES.split_colour_count.get(),
    ) {
        CAR.finish_flag.set(1);
        CAR.started_stand_timer.set(0);
        CAR.notice_id.set(FINISH_NOTICE);
        CAR.notice_time.set(u32::MAX);
        CAR.notice_time_b.set(0);
    }
    Ok(())
}
