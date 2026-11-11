//! `menu::enable_item` (0x46B50F): an item of the current menu made choosable again.
//!
//! The item: +0x48 = 0xFFFFFC1B and its font +0x18 = the one at 0x51E8E4. The twin of
//! `menu::disable_item`, which has the details. Eight callers. Refuses item 0.

use crate::errors::Refused;
use crate::menu::disable_item::set_item;
use crate::menu::owner::Screens;

pub fn enable_item(screens: &Screens, number: u32) -> Result<(), Refused> {
    // The logic is in `rc2k_core::menu::items::choosable`.
    set_item(screens, number, true)
}
