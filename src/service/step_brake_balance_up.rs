//! `service::step_brake_balance_up` (0x46BF90): steps the brake balance, the current car's dword at +0x110, one
//! up, stopping at 100.
//!
//! Returns the current player. The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::CarField(rc2k_core::menu::player::SetupField::BrakeBalance),
    max: 100,
    up: true,
};
