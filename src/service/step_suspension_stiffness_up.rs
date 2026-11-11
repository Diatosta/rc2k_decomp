//! `service::step_suspension_stiffness_up` (0x46BEC4): steps the chosen suspension stiffness, `setups[3].chosen`, one
//! up, stopping at 100.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Chosen(3),
    max: 100,
    up: true,
};
