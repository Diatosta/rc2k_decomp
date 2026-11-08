//! `service::step_tyre_down` (0x46BE3D): steps the chosen tyre, `dword_75AC9C`, one
//! down, stopping at 0.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Global(&crate::menu::owner::MENUS.screens.state.driver_stats.setups[1].chosen),
    max: 5,
    up: false,
};
