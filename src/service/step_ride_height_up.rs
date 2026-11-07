//! `service::step_ride_height_up` (0x46BE8E): steps the chosen ride height, `dword_75ACA8`, one
//! up, stopping at 100.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Global(&crate::menu::owner::MENUS.screens.state.driver_stats.setups[2].chosen),
    max: 100,
    up: true,
};
