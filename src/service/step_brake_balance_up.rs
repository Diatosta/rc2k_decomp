//! Steps the brake balance one up, stopping at 100. The rule at the limits is in `setup_values.rs`
//! (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

pub(super) const STEPPER: Stepper = Stepper {
    target: Target::CarField(rc2k_core::menu::player::SetupField::BrakeBalance),
    max: 100,
    up: true,
};
