//! Steps the ride height one up, stopping at 100. The rule at the limits is in `setup_values.rs`
//! (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

pub(super) const STEPPER: Stepper = Stepper {
    target: Target::Chosen(2),
    max: 100,
    up: true,
};
