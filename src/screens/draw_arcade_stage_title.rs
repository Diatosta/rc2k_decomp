//! `screens::draw_arcade_stage_title` (0x488F2F): the arcade's after-stage screen's title:
//! string 0x1FC at (0x19A, 0x118) with the font at 0x51E8A4. In that screen's draw list
//! (0x758E66).
//!
//!
//! The menus and draw lists run its `entry` (`menu::table_calls`).

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
