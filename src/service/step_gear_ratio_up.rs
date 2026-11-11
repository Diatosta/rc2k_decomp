//! `service::step_gear_ratio_up` (0x46BEFA): steps `setups[0].chosen`, the chosen gear ratio (setup item 0, `menu::state_store`; five pictures on the screen), one
//! up, stopping at 4.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Chosen(0),
    max: 4,
    up: true,
};
