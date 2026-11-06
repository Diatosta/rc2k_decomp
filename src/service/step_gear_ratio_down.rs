//! `service::step_gear_ratio_down` (0x46BEDF): steps `dword_75AC90`, read as the chosen gear ratio (five pictures on the screen; a reading), one
//! down, stopping at 0.
//!
//! In:  nothing
//! Out: every register preserved
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Global(&crate::menu::owner::MENUS.screens.state.driver_stats.setups[0].chosen),
    max: 4,
    up: false,
};
