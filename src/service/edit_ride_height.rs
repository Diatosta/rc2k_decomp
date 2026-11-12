//! `service::edit_ride_height` (0x46BBA2): the keys while the ride height is being edited
//! (edit page 5): step down, step up, confirm.
//!
//! What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::pages::{RIDE_HEIGHT, change_page};
use crate::system::owner::System;

// ---- Rust version ----

pub fn edit_ride_height(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
    confirm: bool,
) -> Result<(), Refused> {
    change_page(menus, system, input, RIDE_HEIGHT, confirm)
}
