//! `menu::list_item_unusable`: whether the highlighted item of a list menu
//! (the list's item table, 16 bytes per item) cannot be chosen: its first byte's bits 6 and 7
//! exactly 0x40 or 0x80, or its kind (low nibble) 5.

use crate::memory::list_handle::ListMenu;

pub fn list_item_unusable(list_state: &crate::menu::list_state::ListState, list: ListMenu) -> bool {
    rc2k_core::menu::list_menu::unusable(list.item(list_state.choice.get()).first())
}
