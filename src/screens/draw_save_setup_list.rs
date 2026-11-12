//! `screens::draw_save_setup_list` (0x46F031): draws the text of the "save setup" screen: a
//! title (string 0x4DC), a label under each column of slots (strings 0x4E3 and 0x4E4) and the
//! slot names (`draw_setup_slot_names`).
//!
//! Its address is in the draw list at 0x759156. That it is the "save" screen is a reading: the
//! screen loop `menu::save_setup_loop`, which also runs the name editor, uses that list. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

// ---- game data ----

pub fn draw_save_setup_list(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::slot_lists::setup_page`.
    let page: u32 = menus.tables.pages.setup_slot_page.get();
    rc2k_core::screens::slot_lists::setup_page(
        &mut crate::screens::game_screen::SlotScreen::new(canvas, o),
        rc2k_core::screens::slot_lists::SAVE_SETUP_TITLE,
        page,
        |slot| {
            crate::screens::game_screen::HeldText::Slot(
                crate::screens::game_screen::SlotNames::Setups,
                slot,
            )
        },
    )
}
