//! `physics::split_by_mode` (0x436B99): the split's figure by the race mode
//! (`dword_62389C`): 2 the car's place (`g_carBlockStore`'s first `place`, 0x70FA40)
//! into +0xA0C; 1 `physics::rank_split_time`; 3 `physics::compare_split_times`; else
//! nothing.
//!
//! Out: as the step leaves them (mode 2: eax the place); no flags read
//!
//! Callers: `physics::take_split`, `physics::take_split_counted`. Refuses what the steps
//! refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::physics::compare_split_times::compare_split_times;
use crate::physics::rank_split_time::rank_split_time;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use rc2k_core::physics::splits::{SplitFigure, split_figure};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::split_figure`.
pub fn split_by_mode(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    match split_figure(shared.vars.run_kind.get()) {
        SplitFigure::Place => {
            shared.car.split_gaps[0].set(CAR_BLOCKS.cars[0].mode2_place.get());
            Ok(())
        }
        SplitFigure::Rank => rank_split_time(race),
        SplitFigure::Compare => compare_split_times(race),
        SplitFigure::Nothing => Ok(()),
    }
}
