//! `screens::draw_stage_info` (0x473DE4): draws the information panel of a stage: a title, the
//! stage heading, then everything in `stage_panel::draw_stage_body` and the stage's record
//! rows.
//!
//! In:  nothing in registers; it works from the section and index of the event's stage
//! Out: carry as `draw_stage_records` leaves it; every register preserved
//! It is not called directly: its address is in the list of draw functions at 0x758D7E, which
//! belongs to the screen loop `sub_473B21`. That loop is called from `sub_4A0E60` and
//! `sub_4AA4D7`, which step through the stages of an event (`inc index`) and show another
//! screen (`sub_472596`) before this one.
//!
//! The stage is the record `find_record(g_recordTable, index, section)`.
//!
//! Screen: confirmed by the owner on 2026-10-10 with a `blank on` test: the stage details
//! screen shown before each stage starts during a championship. The stage selection screen in
//! the menu has the same layout but is drawn by another function.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::records::find_record::find_record_in;
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::stage_panel::{
    BEST_TIMES_PTR_RVA, TITLE_FONT, draw_records_from, draw_stage_body, dword, label, panel_entry,
    table_dword,
};
use crate::text::draw_text::Canvas;

// ---- game data ----

pub const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
pub const INDEX_RVA: usize = rva(0x755958); // index: the stage in its section
pub const SECTION_RVA: usize = rva(0x75595C); // section
pub const MODE_RVA: usize = rva(0x7556D0); // dword_7556D0: 0x13 takes the title from the table
const SECTION_TITLES_RVA: usize = rva(0x75A28A); // dword_75A28A[section]: title string ids

pub const TITLE_X: i32 = 0x2EB;
pub const TITLE_Y: i32 = 0x77;

// ---- Rust version ----

/// The record of the event's stage, and its section.
pub unsafe fn event_stage() -> (u32, u32) {
    unsafe {
        let table = game_addr(RECORD_TABLE_RVA) as u32;
        let section: u32 = game_read(SECTION_RVA);
        let index: u32 = game_read(INDEX_RVA);
        (find_record_in(table, index as i32, section), section)
    }
}

/// The title string of the event's section: from a table in mode 0x13, otherwise from the
/// section's first record.
pub unsafe fn event_title() -> u32 {
    unsafe {
        let table = game_addr(RECORD_TABLE_RVA) as u32;
        let section: u32 = game_read(SECTION_RVA);
        let first = find_record_in(table, 0, section);
        if game_read::<u32>(MODE_RVA) == 0x13 {
            table_dword(SECTION_TITLES_RVA, section)
        } else {
            dword(first.wrapping_add(0x38))
        }
    }
}

/// Returns the carry flag of `draw_stage_records`. The errors are those of the functions it
/// uses: the track pack lookup, the blitter, the formatters and the record rows.
pub unsafe fn draw_stage_info(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        label(canvas, event_title(), TITLE_FONT, TITLE_X, TITLE_Y)?;
        let (record, section) = event_stage();
        draw_stage_heading(record as *const u8, canvas)?;
        // ebx, the markers' palette, still holds the section here.
        draw_stage_body(record, section, false, canvas)?;
        draw_records_from(game_read(BEST_TIMES_PTR_RVA), record, canvas)
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    panel_entry("screens::draw_stage_info", r, &|canvas| unsafe {
        draw_stage_info(canvas)
    })
}

//
// One case: the event's current stage, with the game's own fonts, strings and records. It
// only runs when all of that is there and looks sane, because the original scans the record
// table and follows several pointers without checks.
