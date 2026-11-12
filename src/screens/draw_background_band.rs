//! `screens::draw_background_band` (0x4698C4): draws a band of the background again.
//!
//! Only while `flow.left_service` is 1: sprite 0xA0 of the sprite table (the background picture) at (0,
//! 0) with the clip rectangle's top and bottom at 0xCB and 0x1C6 for that one blit, and put back
//! after it. In 15 lists of screens (the setup and saved game lists, the service area set the
//! switch). What the band covers has not been seen: the name is a reading. The menus and draw lists
//! run its `entry` (`menu::table_calls`).

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
        { menus.screens.flow.left_service.get() } == 1,
    )
}
