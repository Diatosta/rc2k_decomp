//! `service::edit_gear_ratio` (0x46B91A): the keys while the gear ratio (a reading) is being edited
//! (edit page 1): step down, step up, confirm.
//!
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::service::pages::{GEAR_RATIO, change_page};

// ---- Rust version ----

pub fn edit_gear_ratio(confirm: bool) -> Result<(), Refused> {
    change_page(crate::menu::owner::menus(), GEAR_RATIO, confirm)
}
