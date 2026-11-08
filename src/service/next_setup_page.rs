//! `service::next_setup_page` (0x46F191): shows the next page of the setup slot list: adds 1 to
//! `dword_755244`, and from 3 on goes back to 0.
//!
//! The comparison is signed, so a negative page just goes up by one. In two menu tables.

use crate::menu::owner::Menus;

// ---- Rust version ----

pub fn next_setup_page(menus: &Menus) {
    let tables = &menus.tables;
    // Converted: `rc2k_core::service::setups::next_page`.
    tables
        .pages
        .setup_slot_page
        .set(
            rc2k_core::service::setups::next_page(tables.pages.setup_slot_page.get() as i32) as u32,
        );
}
