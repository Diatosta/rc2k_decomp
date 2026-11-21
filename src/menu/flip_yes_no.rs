//! `menu::flip_yes_no`: the "leave?" question's button turned to the other
//! answer.
//!
//! The answer (`pages.yes_no_answer`) `xor` 1. The left and right action of the question's button.

use crate::menu::pages_store::MenuPages;

pub(super) fn flip_yes_no(pages: &MenuPages) -> u32 {
    let answer = rc2k_core::menu::steps::flip(pages.read().yes_no_answer);
    pages.write().yes_no_answer = answer;
    answer
}
