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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::compare_split_times::compare_split_times_in;
use crate::physics::rank_split_time::rank_split_time_in;
use crate::physics::splits::{FIGURE, MODE, g, guarded, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::splits::{SplitFigure, split_figure};

/// The steps on `r`.
// Converted: `rc2k_core::physics::splits::split_figure`.
pub fn split_by_mode_in(r: &mut Regs) -> Result<(), Refused> {
    match split_figure(g(MODE)) {
        SplitFigure::Place => {
            r.eax = g(0x70FA40);
            p(FIGURE, r.eax);
            Ok(())
        }
        SplitFigure::Rank => rank_split_time_in(r),
        SplitFigure::Compare => compare_split_times_in(r),
        SplitFigure::Nothing => Ok(()),
    }
}

/// The Rust version.
pub unsafe fn split_by_mode(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &split_by_mode_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { split_by_mode(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
