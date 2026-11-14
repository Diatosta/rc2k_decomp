//! Race modes other than 2: unless finished, when a checkpoint is past the splits taken
//! (`splits_recorded` below `splits_passed`, unsigned): `physics::split_by_mode`, its figure kept
//! per split, `splits_recorded` made `splits_passed`; at `split_colour_count` the car is finished:
//! `finish_flag` = 1, `started_stand_timer` = 0, the notice 0xC8 (`notice_time` -1, `notice_time_b`
//! 0).
//!
//! Caller: `physics::race_timers`. Refuses (before anything) a split past 0x3F, and what
//! `split_by_mode` refuses.
use crate::errors::Refused;
use crate::physics::split_by_mode::split_by_mode;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::splits::put_split_time;
use crate::race::groups_store::PanelAndOverlay;
use crate::race::league_store::League;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::progress::{FINISH_NOTICE, finished_at, split_due};

// Core: `rc2k_core::physics::progress::{split_due, finished_at}`.
pub fn take_split(
    o: crate::owners::Owners<'static>,
    split_tables: &SplitTables,
    league: &League,
    panel_and_overlay: &PanelAndOverlay,
    shared: &RaceShared,
    car_blocks: &crate::race::blocks_store::CarBlocks,
) -> Result<(), Refused> {
    if shared.car().finish_flag.get() != 0 {
        return Ok(());
    }
    if !split_due(
        shared.car().splits_recorded.get(),
        shared.car().splits_passed.get(),
    )? {
        return Ok(());
    }
    split_by_mode(
        o,
        split_tables,
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
    shared
        .car()
        .splits_recorded
        .set(shared.car().splits_passed.get());
    if finished_at(
        shared.car().splits_recorded.get(),
        split_tables.split_colour_count.get(),
    ) {
        shared.car().finish_flag.set(1);
        shared.car().started_stand_timer.set(0);
        shared.car().notice_id.set(FINISH_NOTICE);
        shared.car().notice_time.set(u32::MAX);
        shared.car().notice_time_b.set(0);
    }
    Ok(())
}
