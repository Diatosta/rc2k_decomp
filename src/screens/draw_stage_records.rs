//! `screens::draw_stage_records` (0x4742BF): draws the two record rows of a stage, at y = 508
//! and y = 531: "best time: <flag> <name> <car> <time>" and
//! "top speed: <flag> <name> <car> <speed>".
//!
//! In:  edi = best-time entry: +0x08 nationality (flag sprite 0x1C + it), +0x10 car number
//!      (see `car_sprite`), +0x14 time in hundredths of a second, +0x20 name
//! Out: carry as the last `draw_text` leaves it; every register preserved
//! The top-speed row comes from the record the game has selected (see `selection`):
//! +0x30 speed in tenths of a mile per hour, +0x54 name, +0x74 nationality, +0x78 car number.
//! The labels are strings 0x34B ("melhor tempo" in the Portuguese build) and 0x34C
//! ("velocidade máxima").
//!
//! Screen: confirmed on the stage selection screen ("Escolha Etapa"), simulation mode, by
//! blanking this hook (2026-10-09): the two rows at the bottom disappear and nothing else.
//! It has three more call sites that have not been looked at.
//!
//! The sprites are drawn with whatever is in ebx as the palette: a pointer into text, or an
//! index. That only works because those sprites do not use a palette; it is reproduced as is.

use crate::errors::{BadInput, Refused};
use crate::framework::{GameHook, RawHook, Regs, SelfTest, game_read, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::screens::stage_panel::PanelScreen;
use crate::screens::{blanked, font_is_loaded, line_buffer, set_line};
use crate::strings::append_time::show_hours;
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::units::units;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use crate::text::glyph_kerning::state_ptr;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_stage_records",
    rva(0x4742BF),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LABEL_FONT: usize = 0x51E5A4;
const TEXT_FONT: usize = 0x51E584;
const TIME_FONT: usize = 0x51E564;

const RECORD_TABLES_RVA: usize = rva(0x735512); // off_735512: pointers to tables of tables
const SELECTED_GROUP_RVA: usize = rva(0x7558C8); // dword_7558C8
const SELECTED_INDEX_RVA: usize = rva(0x7558CC); // dword_7558CC
const SELECTED_SUB_RVA: usize = rva(0x7558D0); // dword_7558D0
const RECORD_SIZE: u32 = 0xA4;

const ROW_1_Y: i32 = 0x1FC;
const ROW_2_Y: i32 = 0x213;

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The game's selection: a group, a table in that group and a record in that table.
#[derive(Clone, Copy, Debug)]
pub struct Selection {
    /// dword_7558C8
    pub group: u32,
    /// dword_7558D0
    pub sub: u32,
    /// `tables[group][sub]`: the address of the table's first record.
    pub table: u32,
    /// `table + index * 0xA4`, with index = dword_7558CC.
    pub record: u32,
}

/// None while the tables are not there; the original then reads through a null pointer.
pub unsafe fn selection() -> Option<Selection> {
    unsafe {
        let group: u32 = game_read(SELECTED_GROUP_RVA);
        let index: u32 = game_read(SELECTED_INDEX_RVA);
        let sub: u32 = game_read(SELECTED_SUB_RVA);
        let tables: u32 = game_read(RECORD_TABLES_RVA + group as usize * 4);
        if tables == 0 {
            return None;
        }
        let table = dword(tables.wrapping_add(sub.wrapping_mul(4)));
        if table == 0 {
            return None;
        }
        Some(Selection {
            group,
            sub,
            table,
            record: table.wrapping_add(index.wrapping_mul(RECORD_SIZE)),
        })
    }
}

unsafe fn selected_record() -> Option<(u32, u32)> {
    unsafe { selection() }.map(|s| (s.record, s.sub))
}

/// False while no record table is selected, in which case the original must not be run.
pub fn has_selected_record() -> bool {
    unsafe { selected_record() }.is_some()
}

// ---- Rust version ----

/// Where the top-speed row's values are in the selected record, and the y its car's sprite
/// is drawn at: the stage panels' (`draw_stage_records`) and the time trial's
/// (`screens::draw_time_trial_records` 0x48DE45, whose car sprite sits on the first row's
/// y, sic).
pub struct Rows {
    pub nationality: u32,
    pub name: u32,
    pub car: u32,
    pub speed: u32,
    pub car_y: i32,
}

pub const STAGE_ROWS: Rows = Rows {
    nationality: 0x74,
    name: 0x54,
    car: 0x78,
    speed: 0x30,
    car_y: ROW_2_Y,
};

pub const TIME_TRIAL_ROWS: Rows = Rows {
    nationality: 0x9C,
    name: 0x7C,
    car: 0xA0,
    speed: 0x2C,
    car_y: ROW_1_Y,
};

/// Returns the carry flag of the last `draw_text`. Refuses a missing record table and what
/// the speed formatter refuses; the fatal errors are the blitter's.
pub unsafe fn draw_stage_records(entry: u32, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe { draw_records_rows(entry, &STAGE_ROWS, canvas) }
}

/// `draw_stage_records` with the top-speed row's places given. Converted:
/// `rc2k_core::stage_panel::records`.
pub unsafe fn draw_records_rows(
    entry: u32,
    rows: &Rows,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    use rc2k_core::stage_panel::{CarRow, Holder};
    unsafe {
        // Without it the original reads through a null pointer after drawing the first row.
        let Some((record, sub)) = selected_record() else {
            return Err(
                BadInput("no record table is selected: the original reads through null").into(),
            );
        };
        let at = |base: u32, o: u32| dword(base.wrapping_add(o));
        let best = Holder {
            nation: at(entry, 8),
            name: entry.wrapping_add(0x20),
            car: at(entry, 0x10),
            figure: at(entry, 0x14),
        };
        let top = Holder {
            nation: at(record, rows.nationality),
            name: record.wrapping_add(rows.name),
            car: at(record, rows.car),
            figure: at(record, rows.speed),
        };
        let car_row = if rows.car_y == ROW_1_Y {
            CarRow::First
        } else {
            CarRow::Second
        };
        // The top speed's flag takes the selection's third value as its palette (ebx).
        rc2k_core::stage_panel::records(
            &mut PanelScreen::new(canvas),
            best,
            top,
            sub,
            car_row,
            show_hours(),
            units(),
        )
    }
}

// ---- hook ----

/// The line buffer as far as these functions use it: the text and the one-character cell of
/// `draw_time_fixed_pitch`.
unsafe fn line_area() -> Vec<u8> {
    unsafe { from_raw_parts(line_buffer(), 0x80) }.to_vec()
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_rows(&HOOK, r, call_original, &STAGE_ROWS);
}

/// The dispatch of a records drawer (this one, `screens::draw_time_trial_records`).
pub fn dispatch_rows(
    hook: &RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    rows: &Rows,
) {
    hook.count_call();
    if blanked(hook.name()) {
        return;
    }
    let entry = r.edi;

    if !shadow() {
        match unsafe { draw_records_rows(entry, rows, &mut GameCanvas) } {
            Ok(too_wide) => {
                r.set_carry(too_wide);
                r.set_zero(!too_wide);
            }
            // The original, run from the start, redraws the same things and meets the same
            // error.
            Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
        }
        return;
    }

    // Shadow: Rust goes through the motions in the line buffer and lists its blitter calls;
    // the original then does it again and really draws.
    let mut canvas = RecordingCanvas::new(true);
    let too_wide = match unsafe { draw_records_rows(entry, rows, &mut canvas) } {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let area_r = unsafe { line_area() };

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let area_o = unsafe { line_area() };

    let ok = o.preserved_except(r, 0)
        && o.carry() == too_wide
        && area_o == area_r
        && unsafe { *state_ptr() } == canvas.state
        && (!recording_works() || calls == canvas.calls);
    if !ok {
        hook.mismatch(|| {
            format!(
                "entry={entry:#x}: original carry={} calls={} rust carry={too_wide} calls={} same_line_buffer={}",
                o.carry(),
                calls.len(),
                canvas.calls.len(),
                area_o == area_r
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----
//
// Uses the game's own fonts, strings and selected record, so it only runs when those are
// there. The blitter's hook swallows the calls; the line buffer and the kerning table are
// restored.

fn self_test() -> (usize, usize) {
    test_rows(&HOOK, call_original, &STAGE_ROWS)
}

/// The self-test of a records drawer.
pub fn test_rows(
    hook: &RawHook,
    call_original: crate::screens::harness::CallOriginal,
    rows: &Rows,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let fonts_loaded = [LABEL_FONT, TEXT_FONT, TIME_FONT]
        .iter()
        .all(|&f| font_is_loaded(f));
    if blob().is_null() || !fonts_loaded || ![0, 0x34B, 0x34C].iter().all(|&id| is_table_id(id)) {
        return t.skip("the string table or the fonts are not loaded");
    }
    if unsafe { selected_record() }.is_none() {
        return t.skip("no record is selected (its table is not loaded)");
    }
    let saved_line = unsafe { from_raw_parts(line_buffer(), 0x200) }.to_vec();
    let saved_state = unsafe { *state_ptr() };

    // (nationality, car number, time, name)
    let entries: [(u32, u32, u32, &[u8]); 4] = [
        (0, 0, 0, b"A\0"),
        (3, 0x18, 12_345, b"TEST DRIVER\0"),
        (7, 0x19, 360_000, b"Another Name\0"),
        (1, 0x1A, 599_999, b"\0"),
    ];
    for (first, second, time, name) in entries {
        let mut entry = [0u8; 0x60];
        entry[8..12].copy_from_slice(&first.to_le_bytes());
        entry[0x10..0x14].copy_from_slice(&second.to_le_bytes());
        entry[0x14..0x18].copy_from_slice(&time.to_le_bytes());
        entry[0x20..0x20 + name.len()].copy_from_slice(name);

        let before = Regs {
            edi: entry.as_ptr() as u32,
            ..Regs::default()
        };
        let mut ro = before;
        let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
        let area_o = unsafe { line_area() };
        let state_o = unsafe { *state_ptr() };

        unsafe { set_line(&saved_line) };
        let mut canvas = RecordingCanvas::new(false);
        let mine = unsafe { draw_records_rows(entry.as_ptr() as u32, rows, &mut canvas) };
        let area_r = unsafe { line_area() };
        unsafe { set_line(&saved_line) };

        let preserved = ro.preserved_except(&before, 0);
        let ok = mine == Ok(ro.carry())
            && area_o == area_r
            && calls_o == canvas.calls
            && state_o == canvas.state
            && preserved;
        t.check(ok, || {
            format!(
                "first={first} second={second:#x} time={time}: original carry={} calls={:?} preserved={preserved} same_line_buffer={}; rust {mine:?} calls={:?}",
                ro.carry(),
                calls_o,
                area_o == area_r,
                canvas.calls
            )
        });
    }

    unsafe { *state_ptr() = saved_state };
    t.finish()
}
