//! `service::step_steering_sensitivity_up` (0x46BF3E): steps the steering sensitivity, the current car's dword at +0x11C, one
//! up, stopping at 100.
//!
//! Returns the current player. The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::CarField(rc2k_core::menu::player::SetupField::SteeringSensitivity),
    max: 100,
    up: true,
};
