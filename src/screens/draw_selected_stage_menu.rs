//! `screens::draw_selected_stage_menu` (0x481A78): the stage panel for the stage the game has
//! selected, on a screen whose titles are menu items. Instead of drawing the group's and the
//! table's titles it stores their string ids in items 0 and 1 of the current menu, and it
//! builds the heading text (string 0x482) in its own buffer without drawing it.
//!
//! In:  nothing in registers; it works from the selection (`draw_stage_records::selection`)
//! Out: carry as `draw_stage_records` leaves it, or clear if the record rows are skipped;
//!      every register preserved
//! Reached through the draw lists at 0x758CF6 (screen loop `sub_48EC30`, a menu entry) and
//! 0x758E9E (screen loop `sub_482099`, six callers).
//!
//! Screen: confirmed by the owner on 2026-10-10 with `blank on
//! screens::draw_selected_stage_menu`: the stage selection screen in the menu ("Escolha
//! Etapa"), whose two titles sit in menu boxes. Which of the two screen loops that was is not
//! known.
//!
//! The record rows come from one of two best-time tables and are skipped while a table is
//! missing (see `stage_panel::draw_records_for_group`).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::game_write;
use crate::screens::draw_stage_records::selection;
use crate::screens::draw_time_trial_stage_info::GROUP_TITLES_RVA;
use crate::screens::stage_panel::{
    GROUP_COPY_RVA, build_second_heading, draw_records_for_group, draw_stage_body, dword,
    menu_item_id, table_dword,
};
use crate::text::draw_text::Canvas;

// ---- Rust version ----

/// What is left in ebx when the markers are drawn: 1 * 0x5C, from addressing menu item 1.
const MARKER_PALETTE: u32 = 0x5C;

/// The string ids this function gives menu items 0 and 1: the group's and the table's titles.
pub unsafe fn menu_titles() -> Option<(u32, u32)> {
    unsafe {
        let sel = selection()?;
        Some((
            table_dword(GROUP_TITLES_RVA, sel.group),
            dword(sel.table.wrapping_add(0x38)),
        ))
    }
}

/// Returns the carry flag. Refuses a missing selection or menu: the original reads or writes
/// through null.
pub unsafe fn draw_selected_stage_menu(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        let (Some(sel), Some((group_title, table_title))) = (selection(), menu_titles()) else {
            return Err(
                BadInput("no record table is selected: the original reads through null").into(),
            );
        };
        let (Some(item_0), Some(item_1)) = (menu_item_id(0), menu_item_id(1)) else {
            return Err(
                BadInput("there is no current menu: the original writes through null").into(),
            );
        };
        (item_0 as *mut u32).write_unaligned(group_title);
        game_write(GROUP_COPY_RVA, sel.group);
        (item_1 as *mut u32).write_unaligned(table_title);

        build_second_heading(sel.record)?;
        draw_stage_body(sel.record, MARKER_PALETTE, true, canvas)?;
        draw_records_for_group(sel.group, sel.record, canvas)
    }
}

//
// The original writes two string ids into whatever menu is current. The test therefore only
// runs where those items already hold exactly these ids, which is on the screen this function
// draws: there the writes change nothing.
