//! `menu::list_item_unusable` (0x414C1B): whether the highlighted item of a list menu
//! (`dword_51D407`, 16 bytes from +0x18) cannot be chosen: its first byte's bits 6 and 7
//! exactly 0x40 or 0x80, or its kind (low nibble) 5.
//!
//! In:  edi the list
//! Out: carry set when it cannot be chosen; every register kept (eax pushed)
//!
//! Callers: `race::replay_viewer` (three), `sub_4144CF` (three), reading carry.

use crate::install::vars::INSTALLER_PAGES;

/// The Rust version: carry.
pub fn list_item_unusable(list: u32) -> Result<bool, crate::errors::BadInput> {
    let at = INSTALLER_PAGES
        .choice
        .get()
        .wrapping_mul(0x10)
        .wrapping_add(list)
        .wrapping_add(0x18);
    // Converted: `rc2k_core::menu::list_menu::unusable`.
    Ok(rc2k_core::menu::list_menu::unusable(
        crate::pointers::GamePtr::new(at).read::<u8>("a list item")?,
    ))
}
