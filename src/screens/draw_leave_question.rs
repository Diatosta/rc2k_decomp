//! `screens::draw_leave_question` (0x46FA75): the "leave?" question's text.
//!
//! In:  nothing
//! Out: as `text::draw_string_by_id` leaves them (carry, eax = 0, ebx, ecx, edx)
//!
//! String 0x537 at (0x197, 0x122) with the font at 0x51E8A4. In the question screen's
//! draw list (0x758CA2).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- Rust version ----

pub unsafe fn draw_leave_question(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    // Converted: `rc2k_core::screens::notes::LEAVE_QUESTION`.
    crate::screens::game_screen::draw_note(canvas, &rc2k_core::screens::notes::LEAVE_QUESTION)
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_leave_question(canvas) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_leave_question", r, &rust)
}
