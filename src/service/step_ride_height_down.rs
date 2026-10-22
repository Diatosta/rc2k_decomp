//! `service::step_ride_height_down` (0x46BE73): steps the chosen ride height, `dword_75ACA8`, one
//! down, stopping at 0.
//!
//! In:  nothing
//! Out: every register preserved
//! The exact rule at the limits is in `setup_values.rs` (`Stepper`).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::rva;
use crate::service::setup_values::{Stepper, Target};

// ---- Rust version: `setup_values::step` with this ----

pub const STEPPER: Stepper = Stepper {
    target: Target::Global(rva(0x75ACA8)),
    max: 100,
    up: false,
};
