//! `physics::split_by_mode` (0x436B99): the split's figure by the race mode
//! (`dword_62389C`): 2 the car's place (`g_carBlockStore`'s first `place`, 0x70FA40)
//! into +0xA0C; 1 `physics::rank_split_time`; 3 `physics::compare_split_times`; else
//! nothing.
//!
//! In:  nothing
//! Out: as the step leaves them (mode 2: eax the place); no flags read
//!
//! Callers: `physics::take_split`, `physics::take_split_counted`.
//! Refuses what the steps refuse.
//! Self-test: `physics/splits.rs` (`test_splits`).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::physics::compare_split_times::compare_split_times;
use crate::physics::rank_split_time::rank_split_time;
use crate::physics::splits::{FIGURE, MODE, g, p};
use rc2k_core::physics::splits::{SplitFigure, split_figure};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::split_figure`.
pub fn split_by_mode() -> Result<(), Refused> {
    match split_figure(g(MODE)) {
        SplitFigure::Place => {
            p(FIGURE, g(0x70FA40));
            Ok(())
        }
        SplitFigure::Rank => rank_split_time(),
        SplitFigure::Compare => compare_split_times(),
        SplitFigure::Nothing => Ok(()),
    }
}
