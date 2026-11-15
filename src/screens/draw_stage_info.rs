//! The information panel of a stage: a title, the stage heading, then everything in
//! `stage_panel::draw_stage_body` and the stage's record rows. Only reached through the draw list
//! of `menu::next_stage_loop`, which the championship runners step through between stages. The
//! stage is the record found by the event's index and section. Seen as the stage details screen
//! shown before each stage of a championship; the stage selection screen in the menu has the same
//! layout but is drawn by another function.

use crate::errors::Refused;
use crate::menu::lists_store::lists;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::records::find_record::find_record_in;
use crate::records::stages_store::RecordRef;
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::stage_panel::{
    TITLE_FONT, draw_records_from, draw_stage_body, label, section_title,
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
    let table = system.records().stage_tables.championship();
    let section: u32 = screens.event.section.get();
    let index: u32 = screens.event.stage_in_section.get();
    Ok((
        find_record_in(&system.records().stage_tables, table, index as i32, section)?,
        section,
    ))
}

/// The title string of the event's section: from a table in mode 0x13, otherwise from the
/// section's first record.
pub fn event_title(menus: &Menus, system: &System) -> Result<u32, crate::errors::BadInput> {
    let table = system.records().stage_tables.championship();
    let section: u32 = menus.screens.event.section.get();
    let first = find_record_in(&system.records().stage_tables, table, 0, section)?;
    if menus.play.champ_view.event_mode.get() == 0x13 {
        section_title(lists(), section)
    } else {
        Ok(first.record(&system.records().stage_tables)?.name())
    }
}

/// Returns whether `draw_stage_records`' last text was too wide for its box. The errors are those
/// of the functions it uses: the track pack lookup, the blitter, the formatters and the record
/// rows.
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

// One case: the event's current stage, with the game's own fonts, strings and records. It
// only runs when all of that is there and looks sane, because it scans the record table and
// follows several pointers without checks.
