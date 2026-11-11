//! `menu::flip_yes_no` (0x46FA52): the "leave?" question's button turned to the other
//! answer.
//!
//! `g_yesNoAnswer` (0x75524C) `xor` 1. The left and right action of the question's button (the menu
//! table at 0x7544D8). The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::menu::owner::Tables;

pub fn flip_yes_no(tables: &Tables) -> u32 {
    // The logic is in `rc2k_core::menu::steps::flip`.
    let answer = rc2k_core::menu::steps::flip(tables.pages.yes_no_answer.get());
    tables.pages.yes_no_answer.set(answer);
    answer
}
