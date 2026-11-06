//! `service::next_setup_page` (0x46F191): shows the next page of the setup slot list: adds 1 to
//! `dword_755244`, and from 3 on goes back to 0.
//!
//! In:  nothing
//! Out: every register preserved
//! The comparison is signed, so a negative page just goes up by one. In two menu tables.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

// ---- Rust version ----

pub fn next_setup_page() {
    // Converted: `rc2k_core::service::setups::next_page`.
    crate::menu::owner::menus()
        .tables
        .pages
        .setup_slot_page
        .set(rc2k_core::service::setups::next_page(
            crate::menu::owner::menus()
                .tables
                .pages
                .setup_slot_page
                .get() as i32,
        ) as u32);
}
