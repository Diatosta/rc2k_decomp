//! `screens::draw_save_setup_list` (0x46F031): draws the text of the "save setup" screen: a
//! title (string 0x4DC), a label under each column of slots (strings 0x4E3 and 0x4E4) and the
//! slot names (`draw_setup_slot_names`).
//!
//! In:  nothing
//! Out: every register preserved; carry clear
//! Its address is in the draw list at 0x759156. That it is the "save" screen is a reading:
//! the screen loop `sub_46EB7A`, which also runs the name editor, uses that list. IDA had no function here until 2026-10-10.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::service::load_setup_slot::SLOT_PAGE_RVA;
use crate::text::draw_text::Canvas;

// ---- game data ----

// ---- Rust version ----

pub unsafe fn draw_save_setup_list(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::slot_lists::setup_page`.
    let page: u32 = unsafe { crate::framework::game_read(SLOT_PAGE_RVA) };
    rc2k_core::screens::slot_lists::setup_page(
        &mut crate::screens::game_screen::SlotScreen::new(canvas),
        rc2k_core::screens::slot_lists::SAVE_SETUP_TITLE,
        page,
        |slot| crate::service::read_setup_slot::slot_name(slot) as u32,
    )
}
