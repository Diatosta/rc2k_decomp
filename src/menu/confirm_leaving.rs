//! `menu::confirm_leaving` (0x46FA90): asks whether to leave.
//!
//! In:  nothing
//! Out: the registers as `menu::leave_question_loop` leaves them; the flags of
//! `cmp g_yesNoAnswer, 1`
//!
//! `g_yesNoAnswer` (0x75524C) = 0; `menu::leave_question_loop` (the "leave?" screen); if that left
//! `dword_758806` at 1: `dword_75555D`, `dword_755561`, `dword_758806` and the answer 0; then with
//! the answer at 1, `dword_75555D` and `dword_755561` = 1. 37 callers (the screen loops that ask
//! before leaving). Rust callers run its `entry` (`routes.rs`); the registers above are the
//! original's, for reference.

use crate::menu::leaving::with_question;

// ---- Rust version ----

/// `ask` puts the question (its screen); the answer (`g_yesNoAnswer`, 1 yes).
pub fn confirm_leaving(ask: &mut dyn FnMut()) -> u32 {
    // Converted: `rc2k_core::menu::leaving::Question` (`begin`, `confirm`).
    with_question(|q, _| q.begin());
    ask();
    with_question(rc2k_core::menu::leaving::Question::confirm)
}
