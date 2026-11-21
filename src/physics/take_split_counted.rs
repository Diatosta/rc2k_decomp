//! Race mode 2, car 0 only: when a checkpoint is past the splits taken (`splits_recorded` below
//! `splits_passed`, unsigned): `physics::flag_current_car_listed`,
//! `physics::split_by_mode`, the figure kept per split; before the last split the notice 0xC9
//! (`notice_time_b` 0x32, `notice_time` 300) and the count-down up by
//! `SplitTables.stage_numbers[1][split]` x 100; `go_stand_timer` = 0; `splits_recorded` made
//! `splits_passed`; at `split_colour_count` finished: `finish_flag` = 1, the notice 0xC8
//! (`notice_time` -1, `notice_time_b` 0).
//!
//! Refuses (before anything) a split past 0x3F, and what the calls refuse.
use crate::errors::Refused;
use crate::physics::flag_current_car_listed::flag_current_car_listed;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::splits::put_split_time;
use crate::race::frame_store::FrameWorkStore;
use crate::race::groups_store::PanelAndOverlay;
use crate::race::league_store::League;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::progress::{
    FINISH_NOTICE, SPLIT_NOTICE, finished_at, split_bonus, split_due,
};

// Core: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub(super) fn take_split_counted(
    o: crate::owners::Owners<'static>,
    physics: &Physics,
    league: &mut League,
    frame_work: &FrameWorkStore,
    panel_and_overlay: &PanelAndOverlay,
    shared: &RaceShared,
    car_blocks: &crate::race::blocks_store::CarBlocks,
) -> Result<(), Refused> {
    let sound = o.sound;
    let physics_store = physics;
    if shared.work.get().current_car.get() != 0 {
        return Ok(());
    }
    if !split_due(
        shared.car().splits_recorded.get(),
        shared.car().splits_passed.get(),
    )? {
        return Ok(());
    }
    { flag_current_car_listed(frame_work, shared.work.get(), sound) }?;
    split_by_mode(
        o,
        &mut physics.split_tables_mut(),
        league,
        panel_and_overlay,
        shared,
        car_blocks,
    )?;
    put_split_time(
        shared.car(),
        shared.car().splits_recorded.get() as usize,
        shared.car().split_gaps[0].get(),
    )?;
    if shared.car().splits_passed.get() < physics_store.split_tables().split_colour_count {
        shared.car().notice_id.set(SPLIT_NOTICE);
        shared.car().notice_time_b.set(0x32);
        shared.car().notice_time.set(300);
        let bonus = split_bonus(
            physics_store.split_tables().stage_numbers
                [1 + shared.car().splits_recorded.get() as usize],
        );
        shared
            .car()
            .time_left
            .set(shared.car().time_left.get().wrapping_add(bonus));
    }
    shared.car().go_stand_timer.set(0);
    shared
        .car()
        .splits_recorded
        .set(shared.car().splits_passed.get());
    if finished_at(
        shared.car().splits_recorded.get(),
        physics_store.split_tables().split_colour_count,
    ) {
        shared.car().finish_flag.set(1);
        shared.car().notice_id.set(FINISH_NOTICE);
        shared.car().notice_time.set(u32::MAX);
        shared.car().notice_time_b.set(0);
    }
    Ok(())
}
