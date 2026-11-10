//! `menu::list_item_unusable` (0x414C1B): whether the highlighted item of a list menu
//! (`dword_51D407`, 16 bytes from +0x18) cannot be chosen: its first byte's bits 6 and 7
//! exactly 0x40 or 0x80, or its kind (low nibble) 5.
//!
//! Callers: `race::replay_viewer` (three), reading carry.

use crate::menu::list_handle::ListMenu;

/// Carry.
pub fn list_item_unusable(list: ListMenu) -> bool {
    // Converted: `rc2k_core::menu::list_menu::unusable`.
    rc2k_core::menu::list_menu::unusable(
        list.item(crate::install::owner::install().pages.choice.get())
            .first(),
    )
}
