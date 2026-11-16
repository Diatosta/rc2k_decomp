//! The load-game screen's preview for a slot with no saved game: the headings and row
//! numbers, "no entry" (string 0x578, "n/d") as the title and every row's name, and "nothing"
//! (string 0x57A, "-") in the six other places of every row. Called by `draw_savegame_preview`.
//! This panel itself has not been seen for an empty slot.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

/// Returns whether the last text drawn was too wide for its box.
pub fn draw_empty_savegame(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    // The logic is in `rc2k_core::screens::savegame_preview::empty`.
    rc2k_core::screens::savegame_preview::empty(
        &mut crate::screens::game_screen::PreviewScreen::new(canvas, o),
    )
}
