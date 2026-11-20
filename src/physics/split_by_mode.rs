//! The split's figure by the race mode (`run_kind`): 2 the car's place (the first car block's
//! `place`) into the place field; 1 `physics::rank_split_time`; 3 `physics::compare_split_times`;
//! otherwise nothing.
//!
//! Refuses what the steps refuse.
use crate::errors::Refused;
use crate::physics::compare_split_times::compare_split_times;
use crate::physics::rank_split_time::rank_split_time;
use crate::physics::split_tables_store::SplitTables;
use crate::race::groups_store::PanelAndOverlay;
use crate::race::league_store::League;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::splits::{SplitFigure, split_figure};

// Core: `rc2k_core::physics::splits::split_figure`.
pub(super) fn split_by_mode(
    o: crate::owners::Owners<'static>,
    split_tables: &mut SplitTables,
    league: &mut League,
    panel_and_overlay: &PanelAndOverlay,
    shared: &RaceShared,
    car_blocks: &crate::race::blocks_store::CarBlocks,
) -> Result<(), Refused> {
    match split_figure(shared.vars.get().run_kind.get()) {
        SplitFigure::Place => {
            shared.car().split_gaps[0].set(car_blocks.cars[0].mode2_place.get());
            Ok(())
        }
        SplitFigure::Rank => rank_split_time(o, split_tables, panel_and_overlay, shared),
        SplitFigure::Compare => {
            compare_split_times(league, shared.work.get(), &shared.state, shared.car())
        }
        SplitFigure::Nothing => Ok(()),
    }
}
