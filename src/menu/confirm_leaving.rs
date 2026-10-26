//! `menu::confirm_leaving` (0x46FA90): asks whether to leave.
//!
//! In:  nothing
//! Out: the registers as `menu::leave_question_loop` leaves them; the flags of
//!      `cmp g_yesNoAnswer, 1`
//!
//! `g_yesNoAnswer` (0x75524C) = 0; `menu::leave_question_loop` (the "leave?" screen);
//! if that left `dword_758806` at 1: `dword_75555D`, `dword_755561`, `dword_758806` and
//! the answer 0; then with the answer at 1, `dword_75555D` and `dword_755561` = 1.
//! 37 callers (the screen loops that ask before leaving).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, rva};

// ---- game data ----

use crate::menu::leaving::with_question;
const QUESTION_RVA: usize = rva(0x47FA94); // menu::leave_question_loop

// ---- Rust version ----

/// `ask` stands for the question's screen.
pub unsafe fn confirm_leaving(r: &mut Regs, ask: &mut dyn FnMut(&mut Regs)) {
    // Converted: `rc2k_core::menu::leaving::Question` (`begin`, `confirm`).
    with_question(|q, _| q.begin());
    ask(r);
    let answer = with_question(|q, l| q.confirm(l));
    r.set_carry(answer < 1);
    r.set_zero(answer == 1);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { confirm_leaving(r, &mut |r| crate::calls::call(QUESTION_RVA, r)) };
    Ok(())
}
