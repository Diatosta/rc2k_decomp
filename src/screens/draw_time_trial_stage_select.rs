//! The time trial's stage selection panel, a copy of `screens::draw_selected_stage_menu` but for
//! its record rows: from the group 1 table or else the time trial best times (a null table is
//! refused), drawn as the time trial's rows (`TIME_TRIAL_ROWS`). Like that panel it stores the
//! group's and the table's title ids in items 0 and 1 of the current menu instead of drawing them,
//! and builds the heading text (string 0x482) without drawing it.

use crate::errors::{BadInput, Refused};
use crate::screens::draw_selected_stage_menu::menu_titles;
use crate::screens::draw_stage_records::selection_in;
use crate::screens::draw_stage_records::{TIME_TRIAL_ROWS, draw_records_rows};
use crate::screens::stage_panel::{build_second_heading, draw_stage_body, menu_item_id};
use crate::text::draw_text::Canvas;

use crate::menu::event_flow::StageTable;
const GROUP_1_TIMES: &StageTable = &crate::saves::best_times::STAGE_TIMES_ALT;
const TIMES: &StageTable = &crate::saves::best_times::TIME_TRIAL_TIMES;

/// Returns whether the record rows' last text was too wide for its box. Refuses a missing selection
/// or menu.
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
        sel.record,
    )?;
    draw_stage_body(o, gfx, sel.record, true, canvas)?;
    let best = crate::screens::stage_panel::best_of(
        table,
        sel.record
            .record(&system.records().stage_tables)?
            .event_slot(),
    )?;
    draw_records_rows(o, system, &best, &TIME_TRIAL_ROWS, canvas)
}

// The function writes two string ids into whatever menu is current. The test therefore
// only runs where those items already hold exactly these ids, which is on the screen this
// function draws: there the writes change nothing.
