//! `menu::item_at` (0x471CEF): item number ebx of the current menu (by handle:
//! `menus_store::MenuId::item`).
//!
//! The current menu is `loop_flags.menu`; its items are 0x5C bytes each after a 4-byte head, so
//! item n is at menu + 4 + (n - 1) * 0x5C. The original counts up to ebx one item at a time
//! and never looks for the end of the menu. 14 callers.
//! Refuses item number 0, for which the original counts through all 32 bits.

use crate::errors::BadInput;
use crate::menu::menus_store::MenuItem;
use crate::menu::owner::Screens;

/// Item `number` (from 1) of the current menu.
pub fn item_at(screens: &Screens, number: u32) -> Result<&'_ MenuItem, BadInput> {
    screens.loop_flags.menu.get()?.item(number)
}
