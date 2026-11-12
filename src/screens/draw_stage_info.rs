//! `screens::draw_stage_info` (0x473DE4): draws the information panel of a stage: a title, the
//! stage heading, then everything in `stage_panel::draw_stage_body` and the stage's record
//! rows.
//!
//! It is not called directly: its address is in the list of draw functions at 0x758D7E, which
//! belongs to the screen loop `menu::next_stage_loop`. That loop is called from
//! `menu::run_championship` and
//! `menu::run_a8_championship`, which step through the stages of an event (`inc index`) and show
//! another screen (`menu::rally_stages_loop`) before this one.
//!
//! The stage is the record `find_record(g_recordTable, index, section)`.
//!
//! Screen: confirmed by the owner on 2026-10-10 with a `blank on` test: the stage details screen
//! shown before each stage starts during a championship. The stage selection screen in the menu has
//! the same layout but is drawn by another function. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::records::find_record::find_record_in;
use crate::records::stages_store::RecordRef;
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::stage_panel::{
    TITLE_FONT, draw_records_from, draw_stage_body, label, table_dword_at,
};
use crate::system::owner::System;
use crate::text::draw_text::Canvas;

// ---- game data ----

pub const TITLE_X: i32 = 0x2EB;
pub const TITLE_Y: i32 = 0x77;

/// The record of the event's stage, and its section.
pub fn event_stage(
    screens: &Screens,
    system: &System,
) -> Result<(RecordRef, u32), crate::errors::BadInput> {
    let table = system.records.stage_tables.championship();
    let section: u32 = screens.event.section.get();
    let index: u32 = screens.event.index.get();
    Ok((find_record_in(table, index as i32, section)?, section))
}

/// The title string of the event's section: from a table in mode 0x13, otherwise from the
/// section's first record.
pub fn event_title(menus: &Menus, system: &System) -> Result<u32, crate::errors::BadInput> {
    let table = system.records.stage_tables.championship();
    let section: u32 = menus.screens.event.section.get();
    let first = find_record_in(table, 0, section)?;
    if menus.play.champ_view.event_mode.get() == 0x13 {
        table_dword_at(
            &menus.tables.lists,
            &menus.tables.lists.section_titles,
            section,
        )
    } else {
        Ok(first.record()?.name())
    }
}

/// Returns the carry flag of `draw_stage_records`. The errors are those of the functions it
/// uses: the track pack lookup, the blitter, the formatters and the record rows.
pub fn draw_stage_info(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    label(
        crate::strings::append_string_by_id::StringOwners::of(o),
        canvas,
        event_title(menus, system)?,
        TITLE_FONT,
        TITLE_X,
        TITLE_Y,
    )?;
    let (record, _) = event_stage(screens, system)?;
    draw_stage_heading(o, record, canvas)?;
    draw_stage_body(o, gfx, record, false, canvas)?;
    draw_records_from(
        o,
        system,
        &crate::saves::best_times::STAGE_TIMES,
        record,
        canvas,
    )
}

//
// One case: the event's current stage, with the game's own fonts, strings and records. It
// only runs when all of that is there and looks sane, because the original scans the record
// table and follows several pointers without checks.
