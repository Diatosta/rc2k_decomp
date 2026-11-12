//! `service::edit_brake_balance` (0x46BCD8): the keys while the brake balance is being
//! edited (edit page 4): step down, step up, and confirm, which leaves edit mode.
//!
//! The value is changed in the player as the keys come. Refuses a missing player.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::pages::{BRAKE_BALANCE, simple_page};
use crate::system::owner::System;

// ---- Rust version ----

pub fn edit_brake_balance(menus: &Menus, system: &System) -> Result<(), Refused> {
    simple_page(menus, system, BRAKE_BALANCE)
}
