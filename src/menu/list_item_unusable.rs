//! `menu::list_item_unusable`: whether the highlighted item of a list menu
//! (the item table, 16 bytes from +0x18) cannot be chosen: its first byte's bits 6 and 7
//! exactly 0x40 or 0x80, or its kind (low nibble) 5.
//!
//! Callers: `race::replay_viewer` (three).

use crate::menu::list_handle::ListMenu;

pub fn list_item_unusable(install: &crate::install::owner::Install, list: ListMenu) -> bool {
    rc2k_core::menu::list_menu::unusable(list.item(install.pages.choice.get()).first())
}
