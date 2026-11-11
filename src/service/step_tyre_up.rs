//! `service::step_tyre_up` (0x46BE58): steps the chosen tyre, `setups[1].chosen`, one
//! up, stopping at 5.
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
            .setups[1]
            .chosen,
    ),
    max: 5,
    up: true,
};
