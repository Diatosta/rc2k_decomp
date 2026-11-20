//! Steps the tyre one up, stopping at 5. The rule at the limits is in `setup_values.rs`
//! (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

pub(super) const STEPPER: Stepper = Stepper {
    target: Target::Chosen(1),
    max: 5,
    up: true,
};
