//! `screens::draw_leave_question` (0x46FA75): the "leave?" question's text.
//!
//! In:  nothing
//! Out: as `text::draw_string_by_id` leaves them (carry, eax = 0, ebx, ecx, edx)
//!
//! String 0x537 at (0x197, 0x122) with the font at 0x51E8A4. In the question screen's draw list
//! (0x758CA2). The menus and draw lists run its `entry` (`menu::table_calls`); the registers above
//! are the original's, for reference.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

// ---- Rust version ----

pub unsafe fn draw_leave_question(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::notes::LEAVE_QUESTION`.
    crate::screens::game_screen::draw_note(canvas, &rc2k_core::screens::notes::LEAVE_QUESTION)?;
    Ok(())
}
