//! `service::step_ride_height_down` (0x46BE73): steps the chosen ride height, `setups[2].chosen`, one
//! down, stopping at 0.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Global(
        &crate::menu::owner::MENUS
            .unchecked()
            .screens
            .state
            .driver_stats
            .setups[2]
            .chosen,
    ),
    max: 100,
    up: false,
};
