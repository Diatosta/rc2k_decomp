//! `menu::ask_to_leave` (0x47E8BB): `dword_755569` 1: the screen loop asks whether to leave
//! (the review menus' "back to the main menu", a reading).
//!
//! An item action in six menu tables (the review menus). The menus and draw lists run its `entry`
//! (`menu::table_calls`).
pub fn ask_to_leave() {
    // Converted: `rc2k_core::menu::leaving::Leaving::ask_to_leave`.
    crate::menu::leaving::with_leaving(rc2k_core::menu::leaving::Leaving::ask_to_leave);
}
