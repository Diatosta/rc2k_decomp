//! `service::step_tyre_down` (0x46BE3D): steps the chosen tyre, `setups[1].chosen`, one
//! down, stopping at 0.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Chosen(1),
    max: 5,
    up: false,
};
