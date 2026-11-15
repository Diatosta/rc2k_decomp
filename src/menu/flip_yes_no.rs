//! `menu::flip_yes_no`: the "leave?" question's button turned to the other
//! answer.
//!
//! The answer (`pages.yes_no_answer`) `xor` 1. The left and right action of the question's
//! button. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::menu::pages_store::MenuPages;

pub fn flip_yes_no(pages: &MenuPages) -> u32 {
    let answer = rc2k_core::menu::steps::flip(pages.yes_no_answer.get());
    pages.yes_no_answer.set(answer);
    answer
}
