//! `menu::enable_item` (0x46B50F): an item of the current menu made choosable again.
//!
//! In:  ebx = the item's number (from 1)
//! Out: every register preserved
//!
//! The item: +0x48 = 0xFFFFFC1B and its font +0x18 = the one at 0x51E8E4. The twin of
//! `menu::disable_item`, which has the details. Eight callers. Refuses item 0.

use crate::errors::Refused;
use crate::menu::disable_item::set_item;

// ---- Rust version ----

pub fn enable_item(number: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::items::choosable`.
    set_item(number, true)
}
