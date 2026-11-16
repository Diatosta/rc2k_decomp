//! `menu::item_at`: item number `n` of the current menu (by handle: `menus_store::MenuId::item`).
//!
//! The current menu is `loop_flags.menu`; its items are 0x5C bytes each after a 4-byte head, so
//! item n is at menu + 4 + (n - 1) * 0x5C. The end of the menu is never looked for.
//! Refuses item number 0.

use crate::errors::BadInput;
use crate::menu::menus_store::MenuItem;
use crate::menu::owner::Menus;

/// Item `number` (from 1) of the current menu.
pub fn item_at(menus: &Menus, number: u32) -> Result<&'_ MenuItem, BadInput> {
    menus
        .screens
        .loop_flags
        .menu
        .get()?
        .item(&menus.tables.menus, number)
}
