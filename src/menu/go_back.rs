//! `menu::go_back` (0x471C82): sets three variables of the menu system: `dword_75555D` = 0,
//! `dword_755561` = 1, `dword_755A0C` = -1.
//!
//! In:  nothing
//! Out: every register preserved
//! It is the action of the back button of several menus and the last step of other actions (seven
//! callers, ten menu tables), so it is read as "leave this menu"; what each of the three variables
//! means has not been worked out. The menus and draw lists run its `entry` (`menu::table_calls`);
//! the registers above are the original's, for reference.

// ---- game data ----

// ---- Rust version ----

pub fn go_back() {
    // Converted: `rc2k_core::menu::leaving::Leaving::go_back`.
    crate::menu::leaving::with_leaving(|l| l.go_back());
}
