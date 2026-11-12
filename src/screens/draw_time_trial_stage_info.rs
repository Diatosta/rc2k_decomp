//! `screens::draw_time_trial_stage_info` (0x487011): the stage panel opened from the service
//! area of a time trial. It works from the stage the game has selected and draws three lines
//! above the panel: the group's title, the table's title and a heading built with string
//! 0x482.
//!
//! Reached through the draw list at 0x758CFE of the screen loop
//! `menu::time_trial_stage_details_loop`, which the service area's "stage details" action
//! `menu::view_stage_details` starts when the event type
//! `event.event_type` is 3 (a time trial); in the other event types that action opens
//! `draw_service_stage_info`.
//!
//!
//!
//! The record rows come from one of two best-time tables and are skipped while a table is missing
//! (see `stage_panel::draw_records_for_group`). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::screens::draw_stage_records::selection_in;
use crate::screens::stage_panel::{
    TITLE_FONT, build_second_heading, draw_records_for_group, draw_stage_body, label,
    table_dword_at,
};
use crate::text::draw_text::{Canvas, draw_text_at};

// ---- game data ----

const TITLE_X: i32 = 0x2EB;
const GROUP_TITLE_Y: i32 = 0x77;
const TABLE_TITLE_Y: i32 = 0x94;
const HEADING_Y: i32 = 0xB1;

/// Returns the carry flag. Refuses a missing selection: the original reads through null.
pub fn draw_time_trial_stage_info(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    race: &crate::race::owner::Race,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let Some(sel) = selection_in(menus, system)? else {
        return Err(
            BadInput("no record table is selected: the original reads through null").into(),
        );
    };
    label(
        crate::strings::append_string_by_id::StringOwners::of(o),
        canvas,
        table_dword_at(
            &menus.play.results,
            &menus.play.results.group_titles,
            sel.group,
        )?,
        TITLE_FONT,
        TITLE_X,
        GROUP_TITLE_Y,
    )?;
    race.shared.state.settings.direction.set(sel.group);
    label(
        crate::strings::append_string_by_id::StringOwners::of(o),
        canvas,
        sel.table.title()?,
        TITLE_FONT,
        TITLE_X,
        TABLE_TITLE_Y,
    )?;

    let heading = build_second_heading(
        crate::strings::append_string_by_id::StringOwners::of(o),
        race,
        sel.record,
    )?;
    draw_text_at(TITLE_FONT, &heading, TITLE_X, HEADING_Y, canvas)?;
    // ebx, the markers' palette, is what draw_text left there: a pointer into the heading.
    draw_stage_body(o, gfx, sel.record, true, canvas)?;
    draw_records_for_group(o, system, sel.group, sel.record, canvas)
}
