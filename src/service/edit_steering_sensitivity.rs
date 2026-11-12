//! `service::edit_steering_sensitivity` (0x46BD0E): the keys while the steering sensitivity is being
//! edited (edit page 3): step down, step up, and confirm, which leaves edit mode.
//!
//! The value is changed in the player as the keys come. Refuses a missing player.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::pages::{STEERING_SENSITIVITY, simple_page};
use crate::system::owner::System;

// ---- Rust version ----

pub fn edit_steering_sensitivity(menus: &Menus, system: &System) -> Result<(), Refused> {
    simple_page(menus, system, STEERING_SENSITIVITY)
}
