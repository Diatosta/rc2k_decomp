//! The title of the arcade's after-stage screen.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

pub fn draw_arcade_stage_title(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::notes::ARCADE_STAGE_TITLE`.
    crate::screens::game_screen::draw_note(
        o,
        canvas,
        &rc2k_core::screens::notes::ARCADE_STAGE_TITLE,
    )?;
    Ok(())
}
