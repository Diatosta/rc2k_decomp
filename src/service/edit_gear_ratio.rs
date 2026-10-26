//! `service::edit_gear_ratio` (0x46B91A): the keys while the gear ratio (a reading) is being edited
//! (edit page 1): step down, step up, confirm.
//!
//! In:  ebx = 1 to confirm without taking a key
//! Out: every register preserved
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).
//! The copy's calls are relocated.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::service::pages::{GEAR_RATIO, change_page};

// ---- Rust version ----

pub fn edit_gear_ratio(confirm: bool) -> Result<(), Refused> {
    change_page(GEAR_RATIO, confirm)
}
