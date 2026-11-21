//! The keys while the suspension stiffness is being edited (edit page 6): step down, step up,
//! confirm. What confirming does, and what is refused, is in `pages.rs` (`change_page`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::pages::{SUSPENSION_STIFFNESS, change_page};
use crate::system::owner::System;

pub(super) fn edit_suspension_stiffness(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
    confirm: bool,
) -> Result<(), Refused> {
    change_page(menus, system, input, SUSPENSION_STIFFNESS, confirm)
}
