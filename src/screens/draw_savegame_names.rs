//! `screens::draw_savegame_names` (0x46D45A): the names of the saved games on the page
//! shown, in two columns of five.
//!
//! In:  nothing
//! Out: every register preserved; carry of the last `draw_text`
//!
//! The ten names of 0x20 bytes from `byte_526BB4` + page (`dword_75523C`) times 0x140, at x = 0xCF
//! for the first five and 0x21C for the others, y = 0xB5, 0xD2, 0xEF, 0x10C, 0x129, with the font
//! at 0x51E8C4. In two draw lists (the load and save screens). The twin of
//! `screens::draw_setup_slot_names`. Refuses a page past the tenth (the list has 100 names). The
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

const NAME_LEN: usize = 0x20;
// ---- Rust version ----

pub fn draw_savegame_names(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::slot_lists::names`.
    let page: u32 = crate::menu::owner::menus().tables.pages.save_page.get();
    let names = crate::pointers::GamePtr::to(&crate::saves::owner::saves().save_games.names);
    rc2k_core::screens::slot_lists::names(
        &mut crate::screens::game_screen::SlotScreen::new(canvas),
        &rc2k_core::screens::slot_lists::SAVED_GAMES,
        page,
        |slot| crate::screens::game_screen::HeldText::Place(names.offset(slot * NAME_LEN as u32)),
    )
}
