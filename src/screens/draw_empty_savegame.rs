//! `screens::draw_empty_savegame` (0x46E2DE): the load-game screen's preview for a slot with
//! no saved game: the headings and row numbers, "no entry" (string 0x578) as the title and as
//! every row's name, and "nothing" (string 0x57A) in the six other places of every row.
//!
//! Called by `draw_savegame_preview`. String 0x578 is "n/d" and string 0x57A is "-" (seen on
//! the rows of `draw_savegame_details` in a screenshot); this panel
//! itself, for an empty slot, has not been seen.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

/// Returns the carry flag of the last text drawn.
pub fn draw_empty_savegame(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    // The logic is in `rc2k_core::screens::savegame_preview::empty`.
    rc2k_core::screens::savegame_preview::empty(
        &mut crate::screens::game_screen::PreviewScreen::new(canvas, o),
    )
}
