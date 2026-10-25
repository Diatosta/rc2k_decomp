//! `service::step_ride_height_up` (0x46BE8E): steps the chosen ride height, `dword_75ACA8`, one
//! up, stopping at 100.
//!
//! In:  nothing
//! Out: every register preserved
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::menu::state_store::MENU_STATE;
use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Global(&MENU_STATE.driver_stats.setups[2].chosen),
    max: 100,
    up: true,
};
