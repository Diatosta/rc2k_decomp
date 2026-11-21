//! The text of the "load setup" screen: a title, a label under each column of slots and the
//! slot names (`draw_setup_slot_names`). That it is the "load" screen is a reading, by elimination
//! from `draw_save_setup_list`.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

// ---- game data ----

pub fn draw_load_setup_list(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::slot_lists::setup_page`.
    let page: u32 = menus.tables.pages.read().setup_slot_page;
    rc2k_core::screens::slot_lists::setup_page(
        &mut crate::screens::game_screen::SlotScreen::new(canvas, o),
        rc2k_core::screens::slot_lists::LOAD_SETUP_TITLE,
        page,
        |slot| {
            crate::screens::game_screen::HeldText::Slot(
                crate::screens::game_screen::SlotNames::Setups,
                slot,
            )
        },
    )
}
