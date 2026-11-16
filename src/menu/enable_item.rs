//! `menu::enable_item`: an item of the current menu made choosable again.
//!
//! The item: +0x48 = 0xFFFFFC1B and its font +0x18 = the plain item font. The twin of
//! `menu::disable_item`, which has the details. Eight callers. Refuses item 0.

use crate::errors::Refused;
use crate::menu::disable_item::set_item;
use crate::menu::owner::Menus;

pub fn enable_item(menus: &Menus, number: u32) -> Result<(), Refused> {
    set_item(menus, number, true)
}
