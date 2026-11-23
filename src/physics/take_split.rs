//! Race modes other than 2: unless finished, when a checkpoint is past the splits taken
//! (`splits_recorded` below `splits_passed`, unsigned): `physics::split_by_mode`, its figure kept
//! per split, `splits_recorded` made `splits_passed`; at `split_colour_count` the car is finished:
//! `finish_flag` = 1, `started_stand_timer` = 0, the notice 0xC8 (`notice_time` -1, `notice_time_b`
//! 0).
//!
//! Refuses (before anything) a split past 0x3F, and what `split_by_mode` refuses.
use crate::errors::Refused;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::splits::put_split_time;
use crate::race::league_store::League;
use crate::race::overlay_store::OverlayBlock;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::progress::{FINISH_NOTICE, finished_at, split_due};

// Core: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub(super) fn take_split(
    o: crate::owners::Owners<'static>,
    split_tables: &mut SplitTables,
    league: &mut League,
    overlay: &std::cell::RefCell<OverlayBlock>,
    shared: &RaceShared,
    car_blocks: &crate::race::blocks_store::CarBlocks,
) -> Result<(), Refused> {
    if shared.car().finish_flag != 0 {
        return Ok(());
    }
    if !split_due(shared.car().splits_recorded, shared.car().splits_passed)? {
        return Ok(());
    }
    split_by_mode(o, split_tables, league, overlay, shared, car_blocks)?;
    let (recorded, gap) = {
        let car = shared.car();
        (car.splits_recorded as usize, car.split_gaps[0])
    };
    put_split_time(&mut shared.car_mut(), recorded, gap)?;
    let value = shared.car().splits_passed;
    shared.car_mut().splits_recorded = value;
    if finished_at(
        shared.car().splits_recorded,
        split_tables.split_colour_count,
    ) {
        shared.car_mut().finish_flag = 1;
        shared.car_mut().started_stand_timer = 0;
        shared.car_mut().notice_id = FINISH_NOTICE;
        shared.car_mut().notice_time = u32::MAX;
        shared.car_mut().notice_time_b = 0;
    }
    Ok(())
}
