//! `service::edit_steering_sensitivity` (0x46BD0E): the keys while the steering sensitivity is being
//! edited (edit page 3): step down, step up, and confirm, which leaves edit mode.
//!
//! In:  nothing
//! Out: every register preserved
//! The value is changed in the player as the keys come. Refuses a missing player.
//! The copy's calls are relocated.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::service::pages::{STEERING_SENSITIVITY, simple_page};

// ---- Rust version ----

pub fn edit_steering_sensitivity() -> Result<(), Refused> {
    simple_page(STEERING_SENSITIVITY)
}
