//! Steps the ride height one down, stopping at 0. The rule at the limits is in `setup_values.rs`
//! (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

pub const STEPPER: Stepper = Stepper {
    target: Target::Chosen(2),
    max: 100,
    up: false,
};
