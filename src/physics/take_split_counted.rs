//! `physics::take_split_counted` (0x436833, race mode 2): for car 0 only, when a
//! checkpoint is past the splits taken (+0xAC below +0x90, unsigned):
//! `physics::keep_split_time`, `physics::flag_current_car_listed`,
//! `physics::split_by_mode`, the figure kept at +0x140 [split]; before the last split
//! the notice 0xC9 (+0x730 0x32, +0x72C 300) and the count-down +0x8C up by
//! `dword_601434[split]` x 100; +0xA4 0; the splits taken made +0x90; at
//! `g_splitColourCount` finished: +0xA8 1, the notice 0xC8 (+0x72C -1, +0x730 0).
//!
//! Caller: `physics::race_timers`. Refuses (before anything) a split past 0x3F, and what
//! the calls refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::physics::flag_current_car_listed::flag_current_car_listed;
use crate::physics::keep_split_time::keep_split_time;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::splits::put_split_time;
use crate::race::owner::Race;
use rc2k_core::physics::progress::{
    FINISH_NOTICE, SPLIT_NOTICE, finished_at, split_bonus, split_due,
};

// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split_counted(race: &Race, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let physics_store = &race.physics;
    if race.shared.work.current_car.get() != 0 {
        return Ok(());
    }
    if !split_due(
        race.shared.car.splits_recorded.get(),
        race.shared.car.splits_passed.get(),
    )? {
        return Ok(());
    }
    { keep_split_time(&race.shared.car, &race.physics.split_tables) }?;
    { flag_current_car_listed(race, sound) }?;
    split_by_mode(race)?;
    put_split_time(
        &race.shared.car,
        race.shared.car.splits_recorded.get() as usize,
        race.shared.car.split_gaps[0].get(),
    )?;
    if race.shared.car.splits_passed.get() < physics_store.split_tables.split_colour_count.get() {
        race.shared.car.notice_id.set(SPLIT_NOTICE);
        race.shared.car.notice_time_b.set(0x32);
        race.shared.car.notice_time.set(300);
        let bonus = split_bonus(
            physics_store.split_tables.stage_numbers
                [1 + race.shared.car.splits_recorded.get() as usize]
                .get(),
        );
        race.shared
            .car
            .time_left
            .set(race.shared.car.time_left.get().wrapping_add(bonus));
    }
    race.shared.car.go_stand_timer.set(0);
    race.shared
        .car
        .splits_recorded
        .set(race.shared.car.splits_passed.get());
    if finished_at(
        race.shared.car.splits_recorded.get(),
        physics_store.split_tables.split_colour_count.get(),
    ) {
        race.shared.car.finish_flag.set(1);
        race.shared.car.notice_id.set(FINISH_NOTICE);
        race.shared.car.notice_time.set(u32::MAX);
        race.shared.car.notice_time_b.set(0);
    }
    Ok(())
}
