//! `menu::confirm_leaving`: asks whether to leave.
//!
//! The yes / no answer is set to 0 and `menu::leave_question_loop` (the "leave?" screen) run; if
//! that left `captions.left_by_key` at 1, `loop_flags.event_left`, `loop_flags.leave_screen`,
//! `captions.left_by_key` and the answer are set to 0; then with the answer at 1,
//! `loop_flags.event_left` and `loop_flags.leave_screen` are set to 1.

use crate::menu::leaving::with_question;
use crate::menu::owner::Menus;

/// `ask` puts the question (its screen). Gives back the answer, 1 for yes.
pub fn confirm_leaving(menus: &Menus, ask: &mut dyn FnMut()) -> u32 {
    with_question(menus, |q, _| q.begin());
    ask();
    with_question(menus, rc2k_core::menu::leaving::Question::confirm)
}
