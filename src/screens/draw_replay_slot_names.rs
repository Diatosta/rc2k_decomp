//! The names of the 20 replay slots of the current page (`pages.replay_page`) in two columns
//! of 10: the save and load replay pages' drawer. Refuses a page past the fifth (the table
//! has 100 names).

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

// ---- game data ----

// FontObjects.objects[41].first_char
pub fn draw_replay_slot_names(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::slot_lists::names`.
    let page = menus.tables.pages.replay_page.get();
    rc2k_core::screens::slot_lists::names(
        &mut crate::screens::game_screen::SlotScreen::new(canvas, o),
        &rc2k_core::screens::slot_lists::REPLAYS,
        page,
        |slot| {
            crate::screens::game_screen::HeldText::Slot(
                crate::screens::game_screen::SlotNames::Replays,
                slot,
            )
        },
    )
    .map(|_| ())
}
