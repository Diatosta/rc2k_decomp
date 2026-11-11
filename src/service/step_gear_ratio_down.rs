//! `service::step_gear_ratio_down` (0x46BEDF): steps `setups[0].chosen`, the chosen gear ratio (setup item 0, `menu::state_store`; five pictures on the screen), one
//! down, stopping at 0.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Chosen(0),
    max: 4,
    up: false,
};
