//! `screens::draw_load_setup_list` (0x46F087): draws the text of the "load setup" screen: a
//! title (string 0x4DD), a label under each column of slots (strings 0x4E3 and 0x4E4) and the
//! slot names (`draw_setup_slot_names`).
//!
//! In:  nothing
//! Out: every register preserved; carry clear
//! Its address is in the draw list at 0x75915E. That it is the "load" screen is a reading: by
//! elimination from `draw_save_setup_list`. IDA had no function here until 2026-10-10. The menus
//! and draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

use crate::errors::Refused;
use crate::menu::pages_store::MENU_PAGES;
use crate::text::draw_text::Canvas;

// ---- game data ----

// ---- Rust version ----

pub fn draw_load_setup_list(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::slot_lists::setup_page`.
    let page: u32 = MENU_PAGES.setup_slot_page.get();
    rc2k_core::screens::slot_lists::setup_page(
        &mut crate::screens::game_screen::SlotScreen::new(canvas),
        rc2k_core::screens::slot_lists::LOAD_SETUP_TITLE,
        page,
        |slot| {
            crate::screens::game_screen::HeldText::Place(
                crate::service::read_setup_slot::slot_name(slot).game_ptr(),
            )
        },
    )
}
