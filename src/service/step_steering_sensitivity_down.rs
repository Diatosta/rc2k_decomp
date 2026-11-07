//! `service::step_steering_sensitivity_down` (0x46BF15): steps the steering sensitivity, the current car's dword at +0x11C, one
//! down, stopping at 0.
//!
//! Returns the current player. The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::CarField(rc2k_core::menu::player::SetupField::SteeringSensitivity),
    max: 100,
    up: false,
};
