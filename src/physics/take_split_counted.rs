//! `physics::take_split_counted` (0x436833, race mode 2): for car 0 only, when a
//! checkpoint is past the splits taken (+0xAC below +0x90, unsigned):
//! `physics::keep_split_time`, `physics::flag_current_car_listed`,
//! `physics::split_by_mode`, the figure kept at +0x140 [split]; before the last split
//! the notice 0xC9 (+0x730 0x32, +0x72C 300) and the count-down +0x8C up by
//! `SplitTables.stage_numbers[1][split]` x 100; +0xA4 0; the splits taken made +0x90; at
//! `g_splitColourCount` finished: +0xA8 1, the notice 0xC8 (+0x72C -1, +0x730 0).
//!
//! Caller: `physics::race_timers`. Refuses (before anything) a split past 0x3F, and what
//! the calls refuse.
use crate::errors::Refused;
use crate::physics::flag_current_car_listed::flag_current_car_listed;
use crate::physics::keep_split_time::keep_split_time;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::splits::put_split_time;
use crate::race::frame_store::FrameWork;
use crate::race::groups_store::PanelAndOverlay;
use crate::race::league_store::League;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::progress::{
    FINISH_NOTICE, SPLIT_NOTICE, finished_at, split_bonus, split_due,
};

// Converted: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split_counted(
    o: crate::owners::Owners<'static>,
    physics: &Physics,
    league: &League,
    frame_work: &FrameWork,
    panel_and_overlay: &PanelAndOverlay,
    shared: &RaceShared,
    car_blocks: &crate::race::blocks_store::CarBlocks,
) -> Result<(), Refused> {
    let sound = o.sound;
    let physics_store = physics;
    if shared.work.current_car.get() != 0 {
        return Ok(());
    }
    if !split_due(
        shared.car.splits_recorded.get(),
        shared.car.splits_passed.get(),
    )? {
        return Ok(());
    }
    { keep_split_time(&shared.car, &physics.split_tables) }?;
    { flag_current_car_listed(frame_work, &shared.work, sound) }?;
    split_by_mode(
        o,
        &physics.split_tables,
        league,
        panel_and_overlay,
        shared,
        car_blocks,
    )?;
    put_split_time(
        &shared.car,
        shared.car.splits_recorded.get() as usize,
        shared.car.split_gaps[0].get(),
    )?;
    if shared.car.splits_passed.get() < physics_store.split_tables.split_colour_count.get() {
        shared.car.notice_id.set(SPLIT_NOTICE);
        shared.car.notice_time_b.set(0x32);
        shared.car.notice_time.set(300);
        let bonus = split_bonus(
            physics_store.split_tables.stage_numbers[1 + shared.car.splits_recorded.get() as usize]
                .get(),
        );
        shared
            .car
            .time_left
            .set(shared.car.time_left.get().wrapping_add(bonus));
    }
    shared.car.go_stand_timer.set(0);
    shared
        .car
        .splits_recorded
        .set(shared.car.splits_passed.get());
    if finished_at(
        shared.car.splits_recorded.get(),
        physics_store.split_tables.split_colour_count.get(),
    ) {
        shared.car.finish_flag.set(1);
        shared.car.notice_id.set(FINISH_NOTICE);
        shared.car.notice_time.set(u32::MAX);
        shared.car.notice_time_b.set(0);
    }
    Ok(())
}
