//! `service::step_brake_balance_down` (0x46BF67): steps the brake balance, the current car's dword at +0x110, one
//! down, stopping at 0.
//!
//! In:  nothing
//! Out: edi = the current player; everything else preserved
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::CarField(0x110),
    max: 100,
    up: false,
};
