//! `screens::draw_time_trial_stage_select` (0x48C2A0; IDA had it as code outside any
//! function, made one 2026-10-19): the time trial's stage selection panel, a copy of
//! `screens::draw_selected_stage_menu` (0x481A78) but for its record rows: from the
//! group 1 table `g_stageBestTimesAlt` (0x7556E4) or else `dword_7556E8`, not looked at for
//! null (refused here), drawn by `screens::draw_time_trial_records` (`TIME_TRIAL_ROWS`).
//! What follows is the copied panel's description:
//!
//! The stage panel for the stage the game has selected, on a screen whose titles are menu
//! items. Instead of drawing the group's and the
//! table's titles it stores their string ids in items 0 and 1 of the current menu, and it
//! builds the heading text (string 0x482) in its own buffer without drawing it.
//!
//! In:  nothing in registers; it works from the selection (`draw_stage_records::selection`)
//! Out: carry as `draw_stage_records` leaves it, or clear if the record rows are skipped;
//! every register preserved
//! Reached through the draw lists at 0x758CF6 (screen loop `sub_48EC30`, a menu entry) and
//! 0x758E9E (screen loop `sub_482099`, six callers).
//!
//! Screen: confirmed by the owner on 2026-10-10 with `blank on
//! screens::draw_selected_stage_menu`: the stage selection screen in the menu ("Escolha
//! Etapa"), whose two titles sit in menu boxes. Which of the two screen loops that was is not
//! known.
//!
//! The record rows come from one of two best-time tables and are skipped while a table is missing
//! (see `stage_panel::draw_records_for_group`). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;
use crate::screens::draw_selected_stage_menu::menu_titles;
use crate::screens::draw_stage_records::selection;
use crate::screens::draw_stage_records::{TIME_TRIAL_ROWS, draw_records_rows};
use crate::screens::stage_panel::{build_second_heading, draw_stage_body, menu_item_id};
use crate::text::draw_text::Canvas;

// ---- Rust version ----

use crate::menu::event_flow::StageTable;
const GROUP_1_TIMES: &StageTable = &crate::saves::best_times::STAGE_TIMES_ALT; // g_stageBestTimesAlt
const TIMES: &StageTable = &crate::saves::best_times::TIME_TRIAL_TIMES; // dword_7556E8

/// Returns the carry flag. Refuses a missing selection or menu: the original reads or writes
/// through null.
pub fn draw_time_trial_stage_select(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let play = &menus().play;
    let screens = &menus().screens;
    let tables = &menus().tables;
    let (Some(sel), Some((group_title, table_title))) = (selection()?, menu_titles(play)?) else {
        return Err(
            BadInput("no record table is selected: the original reads through null").into(),
        );
    };
    let (Some(item_0), Some(item_1)) = (menu_item_id(screens, 0), menu_item_id(screens, 1)) else {
        return Err(BadInput("there is no current menu: the original writes through null").into());
    };
    item_0.set(group_title);
    crate::race::owner::race()
        .shared
        .state
        .settings
        .direction
        .set(sel.group);
    item_1.set(table_title);

    let table = if sel.group == 1 { GROUP_1_TIMES } else { TIMES };
    if !table.is_taken() {
        return Err(BadInput("no time trial best-times table: the original reads near 0").into());
    }
    build_second_heading(sel.record)?;
    draw_stage_body(tables, sel.record, true, canvas)?;
    let best = crate::screens::stage_panel::best_of(table, sel.record.record()?.event_slot())?;
    draw_records_rows(&best, &TIME_TRIAL_ROWS, canvas)
}

//
// The original writes two string ids into whatever menu is current. The test therefore only
// runs where those items already hold exactly these ids, which is on the screen this function
// draws: there the writes change nothing.
