//! The names of the saved games on the page shown (`pages.save_page`), ten in two columns of
//! five. The twin of `screens::draw_setup_slot_names`. Refuses a page past the tenth (the list has
//! 100 names).

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

pub fn draw_savegame_names(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::slot_lists::names`.
    let page: u32 = menus.tables.pages.read().save_page;
    rc2k_core::screens::slot_lists::names(
        &mut crate::screens::game_screen::SlotScreen::new(canvas, o),
        &rc2k_core::screens::slot_lists::SAVED_GAMES,
        page,
        |slot| {
            crate::screens::game_screen::HeldText::Slot(
                crate::screens::game_screen::SlotNames::SavedGames,
                slot,
            )
        },
    )
}
