//! `screens::draw_stage_info` (0x473DE4): draws the information panel of the selected stage:
//! a title, the stage heading, the two map markers, start, finish and total time, length,
//! the four road surface percentages, temperature and wind, the weather line, a three-string
//! description, a numbered line at the bottom left and the stage's record rows.
//!
//! In:  nothing in registers; it works from the selected section and index
//! Out: carry as `draw_stage_records` leaves it; every register preserved
//! It is not called directly: its address is in a table of screen functions at 0x758D7E.
//!
//! The stage is the record `find_record(g_recordTable, index, section)`. Its track pack entry
//! holds most of what is shown: +0x08 index of its duration, +0x0C length in hundredths of a
//! mile (0 = none), +0x10 first of three description strings, +0x14 weather string, +0x1C
//! temperature, +0x20 wind in tenths of a mile per hour, +0x24/+0x28/+0x2C/+0x30 surface
//! percentages, +0x38 a number and +0x3C which of two labels goes with it. The record itself
//! gives +0x10 the start time as hours * 100 + minutes and +0x3C the number of its best-time
//! entry.
//!
//! The layout matches the stage selection screen seen on 2026-10-09 ("partida", "chegada",
//! "total", "extensão", ...), but that screen is drawn by another function with the same
//! layout: what this one is used for is not confirmed.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::blit_glyph::{BlitCall, record_calls, recording_works};
use crate::gfx::blit_marker_pair::{
    marker_calls, marker_position, set_marker_position, sprite_table,
};
use crate::math::hhmm_to_minutes::hhmm_to_minutes;
use crate::records::find_record::{find_record_in, has_section};
use crate::records::trakpak_entry::record_entry;
use crate::screens::draw_stage_heading::draw_stage_heading;
use crate::screens::draw_stage_records::{draw_stage_records, has_selected_record};
use crate::screens::{blanked, font_at, font_is_loaded, line_buffer, set_line};
use crate::strings::append_clock::append_clock;
use crate::strings::append_distance_2dp::append_distance_2dp;
use crate::strings::append_percent::append_percent;
use crate::strings::append_speed::append_speed;
use crate::strings::append_string_by_id::push_string;
use crate::strings::append_temperature::append_temperature;
use crate::strings::append_u32::append_u32;
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text, text_from_ids};
use crate::strings::units::{is_metric, units};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas, draw_text_at};
use crate::text::glyph_kerning::state_ptr;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
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

const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
const INDEX_RVA: usize = rva(0x755958); // index: the selected stage in its section
const SECTION_RVA: usize = rva(0x75595C); // section: the selected section
const MODE_RVA: usize = rva(0x7556D0); // dword_7556D0: 0x13 takes the title from SECTION_TITLES
const SECTION_TITLES_RVA: usize = rva(0x75A28A); // dword_75A28A[section]: title string ids
const DURATIONS_RVA: usize = rva(0x75A49A); // dword_75A49A[entry +8]: minutes
const BEST_TIMES_PTR_RVA: usize = rva(0x7556D8); // dword_7556D8: pointer to 0x618-byte entries
const SCRATCH_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable used all over the game
const FINISH_RVA: usize = rva(0x7559F0); // mmss: receives the finish time

const BEST_TIME_ENTRY_SIZE: u32 = 0x618;

const TITLE_FONT: usize = 0x51E9A4;
const LABEL_FONT: usize = 0x51E5A4;
const VALUE_FONT: usize = 0x51E584;
const HEADING_FONT: usize = 0x51E5C4;
const DESCRIPTION_FONT: usize = 0x51E3E4;
const NUMBER_FONT: usize = 0x51E3C4;
const FONTS: [usize; 6] = [
    TITLE_FONT,
    LABEL_FONT,
    VALUE_FONT,
    HEADING_FONT,
    DESCRIPTION_FONT,
    NUMBER_FONT,
];

// Columns: a label ends at LABEL_x, its value starts at VALUE_x.
const LEFT_LABEL_X: i32 = 0x211;
const LEFT_VALUE_X: i32 = 0x215;
const RIGHT_LABEL_X: i32 = 0x2B8;
const RIGHT_VALUE_X: i32 = 0x2BC;
const CENTRE_X: i32 = 0x266;

const START_LABEL: u32 = 0x34D;
const FINISH_LABEL: u32 = 0x34E;
const TOTAL_LABEL: u32 = 0x356;
const LENGTH_LABEL: u32 = 0x34F;
const LENGTH_UNIT_IMPERIAL: u32 = 0x56F;
const LENGTH_UNIT_METRIC: u32 = 0x570;
const SURFACE_HEADING: u32 = 0x345;
const SURFACE_LABELS: [u32; 4] = [0x346, 0x347, 0x348, 0x349];
const WEATHER_HEADING: u32 = 0x354;
const TEMPERATURE_LABEL: u32 = 0x355;
const WIND_LABEL: u32 = 0x352;
const NUMBER_LABEL_IF_ONE: u32 = 0x358;
const NUMBER_LABEL_OTHERWISE: u32 = 0x359;
const LABELS: [u32; 16] = [
    START_LABEL,
    FINISH_LABEL,
    TOTAL_LABEL,
    LENGTH_LABEL,
    LENGTH_UNIT_IMPERIAL,
    LENGTH_UNIT_METRIC,
    SURFACE_HEADING,
    0x346,
    0x347,
    0x348,
    0x349,
    WEATHER_HEADING,
    TEMPERATURE_LABEL,
    WIND_LABEL,
    NUMBER_LABEL_IF_ONE,
    NUMBER_LABEL_OTHERWISE,
];

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Element `index` of a dword table in game memory.
unsafe fn table_dword(table_rva: usize, index: u32) -> u32 {
    unsafe { dword((game_addr(table_rva) as u32).wrapping_add(index.wrapping_mul(4))) }
}

// ---- Rust version ----

/// A time of day given as hours * 100 + minutes, as "H:MM".
fn time_of_day(hhmm: u32) -> Vec<u8> {
    append_clock(hhmm_to_minutes(hhmm))
}

/// Returns the carry flag of `draw_stage_records`. The errors are those of the functions it
/// uses: the track pack lookup, the blitter, the formatters and the record rows.
pub unsafe fn draw_stage_info(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        let label = |canvas: &mut dyn Canvas, id: u32, font: usize, x: i32, y: i32| {
            draw_string_by_id_at(id, font_at(font), x, y, canvas).map(|_| ())
        };
        // A line built in the line buffer and drawn with the value font.
        let value = |canvas: &mut dyn Canvas, mut bytes: Vec<u8>, x: i32, y: i32| {
            bytes.push(0);
            set_line(&bytes);
            draw_text_at(font_at(VALUE_FONT), line_buffer(), x, y, canvas).map(|_| ())
        };

        let table = game_addr(RECORD_TABLE_RVA) as u32;
        let section: u32 = game_read(SECTION_RVA);
        let index: u32 = game_read(INDEX_RVA);

        // Title: from a table in mode 0x13, otherwise from the section's first record.
        let first = find_record_in(table, 0, section);
        let title = if game_read::<u32>(MODE_RVA) == 0x13 {
            table_dword(SECTION_TITLES_RVA, section)
        } else {
            dword(first.wrapping_add(0x38))
        };
        label(canvas, title, TITLE_FONT, 0x2EB, 0x77)?;

        // The stage: heading and the two markers on the map (ebx, the palette, is `section`).
        let record = find_record_in(table, index as i32, section);
        draw_stage_heading(record as *const u8, canvas)?;
        for call in marker_calls(record as *const u8, section, sprite_table()) {
            set_marker_position(&call);
            canvas.blit_glyph(call)?;
        }

        // Start, finish and total time.
        let start = dword(record.wrapping_add(0x10));
        label(canvas, START_LABEL, LABEL_FONT, LEFT_LABEL_X, 0xD1)?;
        value(canvas, time_of_day(start), LEFT_VALUE_X, 0xD1)?;
        game_write(SCRATCH_RVA, start);

        let entry = record_entry(record as *const u8)?;
        let duration = table_dword(DURATIONS_RVA, dword(entry.wrapping_add(8)));
        let finish = duration.wrapping_add(start);
        game_write(FINISH_RVA, finish);
        value(canvas, time_of_day(finish), RIGHT_VALUE_X, 0xD1)?;
        label(canvas, FINISH_LABEL, LABEL_FONT, RIGHT_LABEL_X, 0xD1)?;

        game_write(SCRATCH_RVA, duration);
        value(canvas, time_of_day(duration), LEFT_VALUE_X, 0xE8)?;
        label(canvas, TOTAL_LABEL, LABEL_FONT, LEFT_LABEL_X, 0xE8)?;

        // Length, with its unit. A length of 0 leaves the line empty.
        let length = dword(entry.wrapping_add(0x0C));
        let mut line = Vec::new();
        if length == 0 {
            line.push(0); // the original writes two NULs here
        } else {
            line = append_distance_2dp(length, is_metric())?;
            let unit = if units() != 0 {
                LENGTH_UNIT_METRIC
            } else {
                LENGTH_UNIT_IMPERIAL
            };
            push_string(&mut line, unit);
        }
        value(canvas, line, RIGHT_VALUE_X, 0xE8)?;
        label(canvas, LENGTH_LABEL, LABEL_FONT, RIGHT_LABEL_X, 0xE8)?;

        // Road surface: four percentages, then their labels.
        label(canvas, SURFACE_HEADING, HEADING_FONT, CENTRE_X, 0x107)?;
        for (field, x, y) in [
            (0x24, LEFT_VALUE_X, 0x120),
            (0x2C, RIGHT_VALUE_X, 0x120),
            (0x28, LEFT_VALUE_X, 0x137),
            (0x30, RIGHT_VALUE_X, 0x137),
        ] {
            value(
                canvas,
                append_percent(dword(entry.wrapping_add(field))),
                x,
                y,
            )?;
        }
        for (id, x, y) in [
            (SURFACE_LABELS[0], LEFT_LABEL_X, 0x120),
            (SURFACE_LABELS[1], RIGHT_LABEL_X, 0x120),
            (SURFACE_LABELS[2], LEFT_LABEL_X, 0x137),
            (SURFACE_LABELS[3], RIGHT_LABEL_X, 0x137),
        ] {
            label(canvas, id, LABEL_FONT, x, y)?;
        }

        // Weather: temperature and wind, then the weather line.
        label(canvas, WEATHER_HEADING, HEADING_FONT, CENTRE_X, 0x156)?;
        let (temperature, _) = append_temperature(dword(entry.wrapping_add(0x1C)) as i32);
        value(canvas, temperature, LEFT_VALUE_X, 0x16F)?;
        label(canvas, TEMPERATURE_LABEL, LABEL_FONT, LEFT_LABEL_X, 0x16F)?;
        let wind = append_speed(dword(entry.wrapping_add(0x20)), true, units())?;
        value(canvas, wind, RIGHT_VALUE_X, 0x16F)?;
        label(canvas, WIND_LABEL, LABEL_FONT, RIGHT_LABEL_X, 0x16F)?;
        label(
            canvas,
            dword(entry.wrapping_add(0x14)),
            VALUE_FONT,
            0x198,
            0x186,
        )?;

        // Description: three consecutive strings, built in the text buffer.
        let description = text_from_ids(dword(entry.wrapping_add(0x10)), 3)?;
        copy_nonoverlapping(description.as_ptr(), text(), description.len());
        draw_text_at(font_at(DESCRIPTION_FONT), text(), 0x198, 0x1BD, canvas)?;

        // The numbered line at the bottom left.
        let number = dword(entry.wrapping_add(0x38));
        game_write(SCRATCH_RVA, number);
        let id = if dword(entry.wrapping_add(0x3C)) == 1 {
            NUMBER_LABEL_IF_ONE
        } else {
            NUMBER_LABEL_OTHERWISE
        };
        let mut line = Vec::new();
        push_string(&mut line, id);
        line.push(b' ');
        line.extend(append_u32(number));
        line.push(0);
        set_line(&line);
        draw_text_at(font_at(NUMBER_FONT), line_buffer(), 0xC0, 0x206, canvas)?;

        // The stage's best time and top speed.
        let best_time = game_read::<u32>(BEST_TIMES_PTR_RVA)
            .wrapping_add(dword(record.wrapping_add(0x3C)).wrapping_mul(BEST_TIME_ENTRY_SIZE));
        draw_stage_records(best_time, canvas)
    }
}

// ---- hook ----

/// Everything this function leaves in game memory, apart from what the blitter draws.
#[derive(PartialEq, Eq)]
struct Leftovers {
    line: Vec<u8>,
    text: Vec<u8>,
    scratch: u32,
    finish: u32,
    marker: (i32, i32),
}

unsafe fn leftovers() -> Leftovers {
    unsafe {
        Leftovers {
            line: from_raw_parts(line_buffer(), 0x80).to_vec(),
            text: from_raw_parts(text(), TEXT_LEN).to_vec(),
            scratch: game_read(SCRATCH_RVA),
            finish: game_read(FINISH_RVA),
            marker: marker_position(),
        }
    }
}

unsafe fn restore(saved: &Leftovers, line: &[u8]) {
    unsafe {
        set_line(line);
        copy_nonoverlapping(saved.text.as_ptr(), text(), TEXT_LEN);
        game_write(SCRATCH_RVA, saved.scratch);
        game_write(FINISH_RVA, saved.finish);
        set_marker_position(&BlitCall {
            glyph: 0,
            x: saved.marker.0,
            y: saved.marker.1,
            palette: 0,
            table: 0,
        });
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if blanked() {
        return;
    }

    if !shadow() {
        match unsafe { draw_stage_info(&mut GameCanvas) } {
            Ok(too_wide) => {
                r.set_carry(too_wide);
                r.set_zero(!too_wide);
            }
            // The original, run from the start, redraws the same things and meets the same
            // error.
            Err(_) => unsafe { call_original(r) },
        }
        return;
    }

    // Shadow: Rust goes through the motions without drawing; the original then does it all
    // again and really draws.
    let mut canvas = RecordingCanvas::new(true);
    let Ok(too_wide) = (unsafe { draw_stage_info(&mut canvas) }) else {
        unsafe { call_original(r) };
        return;
    };
    let left_r = unsafe { leftovers() };

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let left_o = unsafe { leftovers() };

    let ok = o.preserved_except(r, 0)
        && o.carry() == too_wide
        && left_o == left_r
        && unsafe { *state_ptr() } == canvas.state
        && (!recording_works() || calls == canvas.calls);
    if !ok {
        HOOK.mismatch(|| {
            format!(
                "original carry={} calls={} rust carry={too_wide} calls={} same_line={} same_text={} same_globals={}",
                o.carry(),
                calls.len(),
                canvas.calls.len(),
                left_o.line == left_r.line,
                left_o.text == left_r.text,
                (left_o.scratch, left_o.finish, left_o.marker)
                    == (left_r.scratch, left_r.finish, left_r.marker)
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test ----
//
// One case: the stage the game has selected, with the game's own fonts, strings and records.
// It only runs when all of that is there and looks sane, because the original scans the record
// table and follows several pointers without checks. The blitter's hook swallows the calls;
// the buffers, work variables and kerning table are restored.

/// Records looked at when checking that the selected section exists.
const SCAN_LIMIT: u32 = 600;

/// Why the selected stage cannot be used for the test, or None if it can.
unsafe fn unusable_stage() -> Option<&'static str> {
    unsafe {
        let table = game_addr(RECORD_TABLE_RVA) as u32;
        let section: u32 = game_read(SECTION_RVA);
        let index: u32 = game_read(INDEX_RVA);
        if index >= 64 || !has_section(table, section, SCAN_LIMIT) {
            return Some("the selected section or index is not in the record table");
        }
        let first = find_record_in(table, 0, section);
        let title = if game_read::<u32>(MODE_RVA) == 0x13 {
            if section >= 64 {
                return Some("the selected section has no title");
            }
            table_dword(SECTION_TITLES_RVA, section)
        } else {
            dword(first.wrapping_add(0x38))
        };
        let record = find_record_in(table, index as i32, section);
        let Ok(entry) = record_entry(record as *const u8) else {
            return Some("the selected stage is not in the loaded track pack");
        };
        let description = dword(entry.wrapping_add(0x10));
        let ids = [
            title,
            dword(entry),
            dword(entry.wrapping_add(0x14)),
            description,
            description.wrapping_add(1),
            description.wrapping_add(2),
        ];
        if !ids.iter().all(|&id| is_table_id(id)) || dword(entry.wrapping_add(8)) >= 64 {
            return Some("the selected stage's track pack entry does not look like one");
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
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let loaded = !blob().is_null()
        && FONTS.iter().all(|&f| font_is_loaded(f))
        && LABELS.iter().all(|&id| is_table_id(id));
    if !loaded {
        return t.skip("the string table or the fonts are not loaded");
    }
    if let Some(why) = unsafe { unusable_stage() } {
        return t.skip(why);
    }

    let saved = unsafe { leftovers() };
    let saved_line = unsafe { from_raw_parts(line_buffer(), 0x200) }.to_vec();
    let saved_state = unsafe { *state_ptr() };

    let before = Regs::default();
    let mut ro = before;
    let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
    let left_o = unsafe { leftovers() };
    let state_o = unsafe { *state_ptr() };

    unsafe { restore(&saved, &saved_line) };
    let mut canvas = RecordingCanvas::new(false);
    let mine = unsafe { draw_stage_info(&mut canvas) };
    let left_r = unsafe { leftovers() };
    unsafe { restore(&saved, &saved_line) };

    let preserved = ro.preserved_except(&before, 0);
    let ok = mine == Ok(ro.carry())
        && left_o == left_r
        && calls_o == canvas.calls
        && state_o == canvas.state
        && preserved;
    t.check(ok, || {
        let first_difference = calls_o
            .iter()
            .zip(&canvas.calls)
            .position(|(a, b): (&BlitCall, &BlitCall)| a != b);
        format!(
            "original carry={} calls={} preserved={preserved}; rust {mine:?} calls={} first different call={first_difference:?} same_line={} same_text={} same_globals={}",
            ro.carry(),
            calls_o.len(),
            canvas.calls.len(),
            left_o.line == left_r.line,
            left_o.text == left_r.text,
            (left_o.scratch, left_o.finish, left_o.marker)
                == (left_r.scratch, left_r.finish, left_r.marker)
        )
    });

    unsafe { *state_ptr() = saved_state };
    t.finish()
}
