//! The "leave?" question's text.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

pub fn draw_leave_question(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::notes::LEAVE_QUESTION`.
    crate::screens::game_screen::draw_note(o, canvas, &rc2k_core::screens::notes::LEAVE_QUESTION)?;
    Ok(())
}
