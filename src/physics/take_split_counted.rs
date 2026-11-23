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
use crate::race::league_store::League;
use crate::race::overlay_store::OverlayBlock;
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
    overlay: &std::cell::RefCell<OverlayBlock>,
    shared: &RaceShared,
    car_blocks: &crate::race::blocks_store::CarBlocks,
) -> Result<(), Refused> {
    let sound = o.sound;
    let physics_store = physics;
    if shared.work().current_car != 0 {
        return Ok(());
    }
    if !split_due(shared.car().splits_recorded, shared.car().splits_passed)? {
        return Ok(());
    }
    { flag_current_car_listed(frame_work, &shared.work(), sound) }?;
    split_by_mode(
        o,
        &mut physics.split_tables_mut(),
        league,
        overlay,
        shared,
        car_blocks,
    )?;
    let (recorded, gap) = {
        let car = shared.car();
        (car.splits_recorded as usize, car.split_gaps[0])
    };
    put_split_time(&mut shared.car_mut(), recorded, gap)?;
    if shared.car().splits_passed < physics_store.split_tables().split_colour_count {
        shared.car_mut().notice_id = SPLIT_NOTICE;
        shared.car_mut().notice_time_b = 0x32;
        shared.car_mut().notice_time = 300;
        let bonus = split_bonus(
            physics_store.split_tables().stage_numbers[1 + shared.car().splits_recorded as usize],
        );
        let value = shared.car().time_left.wrapping_add(bonus);
        shared.car_mut().time_left = value;
    }
    shared.car_mut().go_stand_timer = 0;
    let value = shared.car().splits_passed;
    shared.car_mut().splits_recorded = value;
    if finished_at(
        shared.car().splits_recorded,
        physics_store.split_tables().split_colour_count,
    ) {
        shared.car_mut().finish_flag = 1;
        shared.car_mut().notice_id = FINISH_NOTICE;
        shared.car_mut().notice_time = u32::MAX;
        shared.car_mut().notice_time_b = 0;
    }
    Ok(())
}
