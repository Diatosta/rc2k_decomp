//! `screens::draw_time_trial_stage_info` (0x487011): the stage panel opened from the service
//! area of a time trial. It works from the stage the game has selected and draws three lines
//! above the panel: the group's title, the table's title and a heading built with string
//! 0x482.
//!
//! In:  nothing in registers; it works from the selection (`draw_stage_records::selection`)
//! Out: carry as `draw_stage_records` leaves it, or clear if the record rows are skipped;
//!      every register preserved
//! Reached through the draw list at 0x758CFE of the screen loop `sub_48F1F4`, which the
//! service area's "stage details" action `sub_47037E` starts when the event type
//! `dword_755954` is 3 (a time trial); in the other event types that action opens
//! `draw_service_stage_info`.
//!
//! Screen: confirmed by the owner on 2026-10-10 with `blank on` for this hook: the stage
//! details opened from the service area of a time trial. Named `draw_selected_stage` until
//! then.
//!
//! The record rows come from one of two best-time tables and are skipped while a table is
//! missing (see `stage_panel::draw_records_for_group`).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_ptr, game_write, rva};
use crate::screens::draw_stage_records::selection;
use crate::screens::font_at;
use crate::screens::stage_panel::{
    GROUP_COPY_RVA, SECOND_HEADING_RVA, TITLE_FONT, build_second_heading, draw_records_for_group,
    draw_stage_body, dword, label, panel_entry, stage_unusable, table_dword,
};
use crate::strings::string_table::is_table_id;
use crate::text::draw_text::{Canvas, draw_text_at};

// ---- game data ----

pub const GROUP_TITLES_RVA: usize = rva(0x758718); // dword_758718[group]: title string ids

const TITLE_X: i32 = 0x2EB;
const GROUP_TITLE_Y: i32 = 0x77;
const TABLE_TITLE_Y: i32 = 0x94;
const HEADING_Y: i32 = 0xB1;

// ---- Rust version ----

/// Returns the carry flag. Refuses a missing selection: the original reads through null.
pub unsafe fn draw_time_trial_stage_info(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        let Some(sel) = selection() else {
            return Err(
                BadInput("no record table is selected: the original reads through null").into(),
            );
        };
        label(
            canvas,
            table_dword(GROUP_TITLES_RVA, sel.group),
            TITLE_FONT,
            TITLE_X,
            GROUP_TITLE_Y,
        )?;
        game_write(GROUP_COPY_RVA, sel.group);
        label(
            canvas,
            dword(sel.table.wrapping_add(0x38)),
            TITLE_FONT,
            TITLE_X,
            TABLE_TITLE_Y,
        )?;

        build_second_heading(sel.record)?;
        let heading = draw_text_at(
            font_at(TITLE_FONT),
            game_ptr::<u8>(SECOND_HEADING_RVA),
            TITLE_X,
            HEADING_Y,
            canvas,
        )?;
        // ebx, the markers' palette, is what draw_text left there: a pointer into the heading.
        draw_stage_body(sel.record, heading.text as u32, true, canvas)?;
        draw_records_for_group(sel.group, sel.record, canvas)
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    panel_entry("screens::draw_time_trial_stage_info", r, &|canvas| unsafe {
        draw_time_trial_stage_info(canvas)
    })
}

/// Why the selected stage cannot be used for a test, or None if it can.
pub unsafe fn selected_stage_unusable() -> Option<&'static str> {
    unsafe {
        let Some(sel) = selection() else {
            return Some("no record table is selected");
        };
        if sel.group >= 16
            || !is_table_id(table_dword(GROUP_TITLES_RVA, sel.group))
            || !is_table_id(dword(sel.table.wrapping_add(0x38)))
        {
            return Some("the selection has no title strings");
        }
        stage_unusable(sel.record)
    }
}
