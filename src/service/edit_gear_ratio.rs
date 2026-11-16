//! The keys while the gear ratio is being edited (edit page 1): step down, step up,
//! confirm. What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::pages::{GEAR_RATIO, change_page};
use crate::system::owner::System;

pub fn edit_gear_ratio(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
    confirm: bool,
) -> Result<(), Refused> {
    change_page(menus, system, input, GEAR_RATIO, confirm)
}
