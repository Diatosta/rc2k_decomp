//! `screens::draw_time_trial_stage_select` (0x48C2A0): the time trial's stage selection panel, a copy of
//! `screens::draw_selected_stage_menu` (0x481A78) but for its record rows: from the
//! group 1 table `g_stageBestTimesAlt` (0x7556E4) or else the time trial best times, not looked at for
//! null (refused here), drawn by `screens::draw_time_trial_records` (`TIME_TRIAL_ROWS`).
//! What follows is the copied panel's description:
//!
//! The stage panel for the stage the game has selected, on a screen whose titles are menu
//! items. Instead of drawing the group's and the
//! table's titles it stores their string ids in items 0 and 1 of the current menu, and it
//! builds the heading text (string 0x482) in its own buffer without drawing it.
//!
//! Reached through the draw lists at 0x758CF6 (screen loop `menu::view_courses_loop`, a menu entry)
//! and 0x758E9E (screen loop `menu::select_stage_loop`, six callers).
//!
//! Screen: confirmed by the owner on 2026-10-10 with `blank on
//! screens::draw_selected_stage_menu`: the stage selection screen in the menu ("Escolha
//! Etapa"), whose two titles sit in menu boxes. Which of the two screen loops that was is not
//! known.
//!
//! The record rows come from one of two best-time tables and are skipped while a table is missing
//! (see `stage_panel::draw_records_for_group`). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::screens::draw_selected_stage_menu::menu_titles;
use crate::screens::draw_stage_records::selection_in;
use crate::screens::draw_stage_records::{TIME_TRIAL_ROWS, draw_records_rows};
use crate::screens::stage_panel::{build_second_heading, draw_stage_body, menu_item_id};
use crate::text::draw_text::Canvas;

use crate::menu::event_flow::StageTable;
const GROUP_1_TIMES: &StageTable = &crate::saves::best_times::STAGE_TIMES_ALT; // g_stageBestTimesAlt
const TIMES: &StageTable = &crate::saves::best_times::TIME_TRIAL_TIMES;

/// Returns the carry flag. Refuses a missing selection or menu: the original reads or writes
/// through null.
pub fn draw_time_trial_stage_select(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    race: &crate::race::owner::Race,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let play = &menus.play;
    let screens = &menus.screens;
    let (Some(sel), Some((group_title, table_title))) = (
        selection_in(menus, system)?,
        menu_titles(menus, system, play)?,
    ) else {
        return Err(
            BadInput("no record table is selected: the original reads through null").into(),
        );
    };
    let (Some(item_0), Some(item_1)) = (
        menu_item_id(screens, &menus.tables.menus, 0),
        menu_item_id(screens, &menus.tables.menus, 1),
    ) else {
        return Err(BadInput("there is no current menu: the original writes through null").into());
    };
    item_0.set(group_title);
    race.shared.state.settings.direction.set(sel.group);
    item_1.set(table_title);

    let table = if sel.group == 1 { GROUP_1_TIMES } else { TIMES };
    if !table.is_taken() {
        return Err(BadInput("no time trial best-times table: the original reads near 0").into());
    }
    build_second_heading(
        crate::strings::append_string_by_id::StringOwners::of(o),
        race,
        sel.record,
    )?;
    draw_stage_body(o, gfx, sel.record, true, canvas)?;
    let best = crate::screens::stage_panel::best_of(table, sel.record.record()?.event_slot())?;
    draw_records_rows(o, system, &best, &TIME_TRIAL_ROWS, canvas)
}

//
// The original writes two string ids into whatever menu is current. The test therefore only
// runs where those items already hold exactly these ids, which is on the screen this function
// draws: there the writes change nothing.
