//! `service::step_gear_ratio_up` (0x46BEFA): steps `dword_75AC90`, the chosen gear ratio (setup item 0, `menu::state_store`; five pictures on the screen), one
//! up, stopping at 4.
//!
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Global(&crate::menu::owner::MENUS.screens.state.driver_stats.setups[0].chosen),
    max: 4,
    up: true,
};
