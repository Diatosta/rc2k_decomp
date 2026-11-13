//! The keys while the tyre is being edited (edit page 2): step down, step up,
//! confirm. What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::pages::{TYRE, change_page};
use crate::system::owner::System;

pub fn edit_tyre(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
    confirm: bool,
) -> Result<(), Refused> {
    change_page(menus, system, input, TYRE, confirm)
}
