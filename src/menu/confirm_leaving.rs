//! `menu::confirm_leaving` (0x46FA90): asks whether to leave.
//!
//! `g_yesNoAnswer` (0x75524C) = 0; `menu::leave_question_loop` (the "leave?" screen); if that left
//! `captions.left_by_key` at 1: `loop_flags.event_left`, `loop_flags.leave_screen`, `captions.left_by_key` and the answer 0; then with
//! the answer at 1, `loop_flags.event_left` and `loop_flags.leave_screen` = 1. 37 callers (the screen loops that ask
//! before leaving). Rust callers run its `entry` (`routes.rs`).

use crate::menu::leaving::with_question;

/// `ask` puts the question (its screen); the answer (`g_yesNoAnswer`, 1 yes).
pub fn confirm_leaving(ask: &mut dyn FnMut()) -> u32 {
    // Converted: `rc2k_core::menu::leaving::Question` (`begin`, `confirm`).
    with_question(|q, _| q.begin());
    ask();
    with_question(rc2k_core::menu::leaving::Question::confirm)
}
