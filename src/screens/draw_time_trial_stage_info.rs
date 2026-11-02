//! `screens::draw_time_trial_stage_info` (0x487011): the stage panel opened from the service
//! area of a time trial. It works from the stage the game has selected and draws three lines
//! above the panel: the group's title, the table's title and a heading built with string
//! 0x482.
//!
//! In:  nothing in registers; it works from the selection (`draw_stage_records::selection`)
//! Out: carry as `draw_stage_records` leaves it, or clear if the record rows are skipped;
//! every register preserved
//! Reached through the draw list at 0x758CFE of the screen loop `sub_48F1F4`, which the
//! service area's "stage details" action `sub_47037E` starts when the event type
//! `dword_755954` is 3 (a time trial); in the other event types that action opens
//! `draw_service_stage_info`.
//!
//! Named `draw_selected_stage` until then.
//!
//! The record rows come from one of two best-time tables and are skipped while a table is missing
//! (see `stage_panel::draw_records_for_group`). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::results_store::RESULT_FIGURES;
use crate::race::state_store::RACE_STATE;
use crate::screens::draw_stage_records::selection;
use crate::screens::stage_panel::{
    TITLE_FONT, build_second_heading, draw_records_for_group, draw_stage_body, dword, label,
    table_dword_at,
};
use crate::session::Dword;
use crate::text::draw_text::{Canvas, draw_text_at};

// ---- game data ----

pub const GROUP_TITLES: &[Dword] = &RESULT_FIGURES.group_titles; // dword_758718[group]: title string ids

const TITLE_X: i32 = 0x2EB;
const GROUP_TITLE_Y: i32 = 0x77;
const TABLE_TITLE_Y: i32 = 0x94;
const HEADING_Y: i32 = 0xB1;

// ---- Rust version ----

/// Returns the carry flag. Refuses a missing selection: the original reads through null.
pub fn draw_time_trial_stage_info(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let Some(sel) = selection()? else {
        return Err(
            BadInput("no record table is selected: the original reads through null").into(),
        );
    };
    label(
        canvas,
        table_dword_at(&RESULT_FIGURES, GROUP_TITLES, sel.group)?,
        TITLE_FONT,
        TITLE_X,
        GROUP_TITLE_Y,
    )?;
    RACE_STATE.settings.direction.set(sel.group);
    label(
        canvas,
        dword(sel.table.wrapping_add(0x38))?,
        TITLE_FONT,
        TITLE_X,
        TABLE_TITLE_Y,
    )?;

    build_second_heading(sel.record)?;
    let heading = draw_text_at(
        TITLE_FONT.game_ptr(),
        crate::pointers::GamePtr::to(
            &crate::screens::text_buffers_store::TEXT_BUFFERS.stage_heading,
        ),
        TITLE_X,
        HEADING_Y,
        canvas,
    )?;
    // ebx, the markers' palette, is what draw_text left there: a pointer into the heading.
    draw_stage_body(sel.record, heading.text.addr(), true, canvas)?;
    draw_records_for_group(sel.group, sel.record, canvas)
}
