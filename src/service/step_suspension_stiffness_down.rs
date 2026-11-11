//! `service::step_suspension_stiffness_down` (0x46BEA9): steps the chosen suspension stiffness, `setups[3].chosen`, one
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
            .setups[3]
            .chosen,
    ),
    max: 100,
    up: false,
};
