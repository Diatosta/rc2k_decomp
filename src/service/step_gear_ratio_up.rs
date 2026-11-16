//! Steps the gear ratio one up, stopping at 4. The rule at the limits is in `setup_values.rs`
//! (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

pub const STEPPER: Stepper = Stepper {
    target: Target::Chosen(0),
    max: 4,
    up: true,
};
