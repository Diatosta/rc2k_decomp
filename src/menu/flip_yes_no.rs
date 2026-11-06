//! `menu::flip_yes_no` (0x46FA52): the "leave?" question's button turned to the other
//! answer.
//!
//! In:  nothing
//! Out: every register preserved; the flags of the `xor`
//!
//! `g_yesNoAnswer` (0x75524C) `xor` 1. The left and right action of the question's button (the menu
//! table at 0x7544D8). The menus and draw lists run its `entry` (`menu::table_calls`); the
//! registers above are the original's, for reference.

// ---- Rust version ----

pub fn flip_yes_no() -> u32 {
    // Converted: `rc2k_core::menu::steps::flip`.
    let answer =
        rc2k_core::menu::steps::flip(crate::menu::owner::menus().tables.pages.yes_no_answer.get());
    crate::menu::owner::menus()
        .tables
        .pages
        .yes_no_answer
        .set(answer);
    answer
}
