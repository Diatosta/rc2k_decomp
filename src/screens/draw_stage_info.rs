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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::records::find_record::{find_record_in, has_section};
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::draw_stage_records::has_selected_record;
use crate::screens::stage_panel::{
    BEST_TIMES_PTR_RVA, TITLE_FONT, body_unavailable, check_panel, dispatch_panel,
    draw_records_from, draw_stage_body, dword, label, stage_unusable, table_dword,
};
use crate::strings::string_table::is_table_id;
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_stage_info", rva(0x473DE4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_panel(&HOOK, r, call_original, &|canvas| unsafe {
        draw_stage_info(canvas)
    });
}

// ---- self-test ----
//
// One case: the event's current stage, with the game's own fonts, strings and records. It
// only runs when all of that is there and looks sane, because the original scans the record
// table and follows several pointers without checks.

/// Records looked at when checking that the section exists.
const SCAN_LIMIT: u32 = 600;

/// Why the event's stage cannot be used for a test, or None if it can.
pub unsafe fn event_stage_unusable() -> Option<&'static str> {
    unsafe {
        let table = game_addr(RECORD_TABLE_RVA) as u32;
        let section: u32 = game_read(SECTION_RVA);
        let index: u32 = game_read(INDEX_RVA);
        if index >= 64 || section >= 64 || !has_section(table, section, SCAN_LIMIT) {
            return Some("the event's section or index is not in the record table");
        }
        if !is_table_id(event_title()) {
            return Some("the event's section has no title string");
        }
        if let Some(why) = stage_unusable(event_stage().0) {
            return Some(why);
        }
        if game_read::<u32>(BEST_TIMES_PTR_RVA) == 0 || !has_selected_record() {
            return Some("the best-time or record tables are not loaded");
        }
        None
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = body_unavailable().or_else(|| unsafe { event_stage_unusable() }) {
        return t.skip(why);
    }
    check_panel(&mut t, call_original, &|canvas| unsafe {
        draw_stage_info(canvas)
    });
    t.finish()
}
