//! Steps the tyre one down, stopping at 0. The rule at the limits is in `setup_values.rs`
//! (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

pub(super) const STEPPER: Stepper = Stepper {
    target: Target::Chosen(1),
    max: 5,
    up: false,
};
