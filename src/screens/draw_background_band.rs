//! Draws a band of the background again, only while `flow.left_service` is 1: the background
//! sprite with the clip rectangle narrowed to the band for that one blit. Used by the setup and
//! saved game screens. What the band covers has not been seen; the name is a reading.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

// ---- game data ----

pub fn draw_background_band(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::controller_page::background_band`.
    use rc2k_core::screens::controller_page::{Font, Work, background_band};
    background_band(
        &mut crate::screens::game_screen::GameScreen::<Font, Work>::new(canvas, o),
        { menus.screens.flow.read().left_service } == 1,
    )
}
