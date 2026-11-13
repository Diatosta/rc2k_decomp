//! `menu::flip_yes_no`: the "leave?" question's button turned to the other
//! answer.
//!
//! The answer (`pages.yes_no_answer`) `xor` 1. The left and right action of the question's
//! button. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::menu::owner::Tables;

pub fn flip_yes_no(tables: &Tables) -> u32 {
    let answer = rc2k_core::menu::steps::flip(tables.pages.yes_no_answer.get());
    tables.pages.yes_no_answer.set(answer);
    answer
}
