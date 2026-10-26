//! `menu::next_savegame_page` (0x46D407): the next page of the saved games' list: adds
//! 1 to `dword_75523C`, and above 9 goes back to 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The comparison is signed. In two menu tables (0x7518D0, 0x751D28): the load and save screens'
//! page buttons. The twin of `service::next_setup_page`. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::menu::pages_store::MENU_PAGES;

// ---- Rust version ----

pub fn next_savegame_page() {
    // Converted: `rc2k_core::menu::steps::savegame_page`.
    let page = MENU_PAGES.save_page.get();
    MENU_PAGES
        .save_page
        .set(rc2k_core::menu::steps::savegame_page(page, true));
}
