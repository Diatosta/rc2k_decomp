//! `screens::draw_replay_slot_names` (0x46F41A): draws the names of the 20 replay slots of
//! the current page in two columns of 10 (the save and load replay pages' drawer).
//!
//! The page is `pages.replay_page`; a name is 0x20 bytes of the picture path buffer. Columns at x = 0xC6 and
//! 0x213, rows from y = 0xB5 in steps of 0x1B, font 0x51E8C4. It keeps its position in
//! `g_markerX`/`g_markerY` and its row counter in `draw_work.figure`, left at 0x213, 0x183 and 10.
//! Refuses a page past the fifth: the table has 100 names. Called through the menu tables
//! (0x758BCE, 0x758BD6). The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::name_lists::{NAME_LEN, REPLAY_PAGE_RVA};
use crate::race::owner::race;
use crate::text::draw_text::Canvas;

// ---- game data ----

// g_markerX
// g_markerY
// `draw_work.figure`: a work variable, here the row counter

// stru_51E8C4
/// The name of replay slot `slot`.
fn slot_name(slot: u32) -> crate::place::BytePlace {
    crate::place::BytePlace::In(&race().shared.replays.names, slot as usize * NAME_LEN)
}

pub fn draw_replay_slot_names(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::slot_lists::names`.
    let page = REPLAY_PAGE_RVA.get();
    rc2k_core::screens::slot_lists::names(
        &mut crate::screens::game_screen::SlotScreen::new(canvas),
        &rc2k_core::screens::slot_lists::REPLAYS,
        page,
        |slot| crate::screens::game_screen::HeldText::Place(slot_name(slot).game_ptr()),
    )
    .map(|_| ())
}
