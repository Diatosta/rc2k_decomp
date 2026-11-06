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
//! Caller: `physics::race_timers`. Refuses (before anything) a split past 0x3F, and what
//! the calls refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::keep_split_time::keep_split_time;
use crate::physics::set_flag_51659c::set_flag_51659c;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::splits::put_split_time;
use crate::race::owner::Race;
use crate::race::work_store::RACE_WORK;
use rc2k_core::physics::progress::{
    FINISH_NOTICE, SPLIT_NOTICE, finished_at, split_bonus, split_due,
};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split_counted(race: &Race) -> Result<(), Refused> {
    let physics_store = &race.physics;
    if RACE_WORK.current_car.get() != 0 {
        return Ok(());
    }
    if !split_due(CAR.splits_recorded.get(), CAR.splits_passed.get())? {
        return Ok(());
    }
    { keep_split_time(&race.physics) }?;
    { set_flag_51659c(&race.frame) }?;
    split_by_mode(race)?;
    put_split_time(CAR.splits_recorded.get() as usize, CAR.split_gaps[0].get())?;
    if CAR.splits_passed.get() < physics_store.split_tables.split_colour_count.get() {
        CAR.notice_id.set(SPLIT_NOTICE);
        CAR.notice_time_b.set(0x32);
        CAR.notice_time.set(300);
        let bonus = split_bonus(
            physics_store.split_tables.stage_numbers[1 + CAR.splits_recorded.get() as usize].get(),
        );
        CAR.time_left.set(
            crate::physics::car_store::CAR
                .time_left
                .get()
                .wrapping_add(bonus),
        );
    }
    CAR.go_stand_timer.set(0);
    CAR.splits_recorded.set(CAR.splits_passed.get());
    if finished_at(
        CAR.splits_recorded.get(),
        physics_store.split_tables.split_colour_count.get(),
    ) {
        CAR.finish_flag.set(1);
        CAR.notice_id.set(FINISH_NOTICE);
        CAR.notice_time.set(u32::MAX);
        CAR.notice_time_b.set(0);
    }
    Ok(())
}
