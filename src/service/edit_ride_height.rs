//! `service::edit_ride_height` (0x46BBA2): the keys while the ride height is being edited
//! (edit page 5): step down, step up, confirm.
//!
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::service::pages::{RIDE_HEIGHT, change_page};

// ---- Rust version ----

pub fn edit_ride_height(confirm: bool) -> Result<(), Refused> {
    change_page(crate::menu::owner::menus(), RIDE_HEIGHT, confirm)
}
