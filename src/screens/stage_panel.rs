//! What the four stage panel functions share: `draw_stage_info`, `draw_service_stage_info`,
//! `draw_time_trial_stage_info` and `draw_selected_stage_menu`. In the original each is a full copy of
//! the same layout; they differ only in where the stage comes from, what goes above the panel
//! and which best-time table the record rows use.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    GameHook, RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, rva, shadow,
};
use crate::gfx::blit_glyph::{BlitCall, record_calls, recording_works};
use crate::gfx::blit_marker_pair::{
    marker_calls, marker_position, set_marker_position, sprite_table,
};
use crate::math::hhmm_to_minutes::hhmm_to_minutes;
use crate::records::trakpak_entry::record_entry;
use crate::screens::draw_stage_heading::heading_text;
use crate::screens::draw_stage_records::draw_stage_records;
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

// ---- game data ----

pub const DURATIONS_RVA: usize = rva(0x75A49A); // dword_75A49A[entry +8]: minutes
pub const BEST_TIMES_PTR_RVA: usize = rva(0x7556D8); // dword_7556D8: 0x618-byte entries
pub const GROUP_1_BEST_TIMES_PTR_RVA: usize = rva(0x7556E0); // dword_7556E0: same, for group 1
const SCRATCH_RVA: usize = rva(0x7559EC); // dword_7559EC: a work variable used all over the game
const FINISH_RVA: usize = rva(0x7559F0); // mmss: receives the finish time
const ENTRY_34_RVA: usize = rva(0x75ACD0 + 4); // dword_75ACD0[1]: copy of entry +0x34
pub const GROUP_COPY_RVA: usize = rva(0x75ACF0); // dword_75ACF0: copy of the selected group
pub const SECOND_HEADING_RVA: usize = rva(0xA8BD54); // byte_A8BD54: the 0x482 heading text
pub const MENU_PTR_RVA: usize = rva(0x75558D); // dword_75558D: the current menu

pub const BEST_TIME_ENTRY_SIZE: u32 = 0x618;
const SECOND_HEADING_STRING: u32 = 0x482;

pub const TITLE_FONT: usize = 0x51E9A4;
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
const LABELS: [u32; 17] = [
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
    SECOND_HEADING_STRING,
];

pub unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// Element `index` of a dword table in game memory.
pub unsafe fn table_dword(table_rva: usize, index: u32) -> u32 {
    unsafe { dword((game_addr(table_rva) as u32).wrapping_add(index.wrapping_mul(4))) }
}

// ---- the shared parts ----

/// A time given as hours * 100 + minutes, as "H:MM".
fn time_of_day(hhmm: u32) -> Vec<u8> {
    append_clock(hhmm_to_minutes(hhmm))
}

/// Draws a string id with one of the game's font objects.
pub unsafe fn label(
    canvas: &mut dyn Canvas,
    id: u32,
    font: usize,
    x: i32,
    y: i32,
) -> Result<(), Refused> {
    unsafe { draw_string_by_id_at(id, font_at(font), x, y, canvas) }.map(|_| ())
}

/// Builds the second heading ("<string 0x482> n/6: name") in its own buffer. Three of the
/// four panels build it; only `draw_time_trial_stage_info` also draws it.
pub unsafe fn build_second_heading(record: u32) -> Result<(), FatalError> {
    unsafe {
        let bytes = heading_text(record as *const u8, SECOND_HEADING_STRING)?;
        copy_nonoverlapping(
            bytes.as_ptr(),
            game_ptr::<u8>(SECOND_HEADING_RVA),
            bytes.len(),
        );
    }
    Ok(())
}

/// Everything from the map markers down to the numbered line at the bottom left, for the
/// stage `record`. `marker_palette` is whatever the caller has in ebx when the markers are
/// drawn. `keep_entry_34` is the one store that two of the panels add: entry +0x34 into
/// dword_75ACD0[1].
///
/// The stage's track pack entry holds most of what is shown: +0x08 index of its duration,
/// +0x0C length in hundredths of a mile (0 = none), +0x10 first of three description strings,
/// +0x14 weather string, +0x1C temperature, +0x20 wind in tenths of a mile per hour,
/// +0x24/+0x28/+0x2C/+0x30 surface percentages, +0x38 a number and +0x3C which of two labels
/// goes with it. The record gives +0x10, the start time as hours * 100 + minutes.
pub unsafe fn draw_stage_body(
    record: u32,
    marker_palette: u32,
    keep_entry_34: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe { draw_stage_body_in(record, marker_palette, keep_entry_34, HEADING_FONT, canvas) }
}

/// `draw_stage_body` with the two headings (surface, weather) in `heading_font`: the rally
/// stage page (`screens::draw_rally_stage_info`) uses `stru_51E564`, the others
/// `stru_51E5C4`.
pub unsafe fn draw_stage_body_in(
    record: u32,
    marker_palette: u32,
    keep_entry_34: bool,
    heading_font: usize,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        // A line built in the line buffer and drawn with the value font.
        let value = |canvas: &mut dyn Canvas, mut bytes: Vec<u8>, x: i32, y: i32| {
            bytes.push(0);
            set_line(&bytes);
            draw_text_at(font_at(VALUE_FONT), line_buffer(), x, y, canvas).map(|_| ())
        };

        // The two markers on the map.
        for call in marker_calls(record as *const u8, marker_palette, sprite_table()) {
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
        if keep_entry_34 {
            game_write(ENTRY_34_RVA, dword(entry.wrapping_add(0x34)));
        }

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
        label(canvas, SURFACE_HEADING, heading_font, CENTRE_X, 0x107)?;
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
        label(canvas, WEATHER_HEADING, heading_font, CENTRE_X, 0x156)?;
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
        Ok(())
    }
}

/// The record rows, with the stage's best-time entry taken from the table at `table`.
/// Returns the carry flag.
pub unsafe fn draw_records_from(
    table: u32,
    record: u32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    unsafe {
        let entry = dword(record.wrapping_add(0x3C)).wrapping_mul(BEST_TIME_ENTRY_SIZE);
        draw_stage_records(table.wrapping_add(entry), canvas)
    }
}

/// The record rows as the two "selected stage" panels draw them: from the group 1 table when
/// group 1 is selected, else from the usual one, and not at all while either the group 1
/// table or the chosen one is missing. Returns the carry flag (clear when nothing is drawn).
pub unsafe fn draw_records_for_group(
    group: u32,
    record: u32,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    unsafe {
        let mut table: u32 = game_read(GROUP_1_BEST_TIMES_PTR_RVA);
        if table == 0 {
            return Ok(false);
        }
        if group != 1 {
            table = game_read(BEST_TIMES_PTR_RVA);
        }
        if table == 0 {
            return Ok(false);
        }
        draw_records_from(table, record, canvas)
    }
}

// ---- hook and self-test support ----

/// `call_original` of a hook.
pub type CallOriginal = unsafe extern "C" fn(*mut Regs);

/// Address of the string id of item `n` of the current menu, or None without a menu.
pub unsafe fn menu_item_id(n: u32) -> Option<u32> {
    let menu: u32 = unsafe { game_read(MENU_PTR_RVA) };
    (menu != 0).then(|| menu.wrapping_add(4 + n * 0x5C + 0x14))
}

/// Everything the panels leave in game memory, apart from what the blitter draws.
#[derive(PartialEq, Eq)]
pub struct Leftovers {
    line: Vec<u8>,
    second_heading: Vec<u8>,
    text: Vec<u8>,
    scratch: u32,
    finish: u32,
    entry_34: u32,
    group_copy: u32,
    marker: (i32, i32),
    menu_ids: (u32, u32),
}

impl Leftovers {
    pub unsafe fn read() -> Self {
        unsafe {
            let menu_id = |n| menu_item_id(n).map_or(0, |at| dword(at));
            Self {
                line: from_raw_parts(line_buffer(), 0x200).to_vec(),
                second_heading: from_raw_parts(game_ptr::<u8>(SECOND_HEADING_RVA), 0x100).to_vec(),
                text: from_raw_parts(text(), TEXT_LEN).to_vec(),
                scratch: game_read(SCRATCH_RVA),
                finish: game_read(FINISH_RVA),
                entry_34: game_read(ENTRY_34_RVA),
                group_copy: game_read(GROUP_COPY_RVA),
                marker: marker_position(),
                menu_ids: (menu_id(0), menu_id(1)),
            }
        }
    }

    pub unsafe fn restore(&self) {
        unsafe {
            set_line(&self.line);
            copy_nonoverlapping(
                self.second_heading.as_ptr(),
                game_ptr::<u8>(SECOND_HEADING_RVA),
                self.second_heading.len(),
            );
            copy_nonoverlapping(self.text.as_ptr(), text(), TEXT_LEN);
            game_write(SCRATCH_RVA, self.scratch);
            game_write(FINISH_RVA, self.finish);
            game_write(ENTRY_34_RVA, self.entry_34);
            game_write(GROUP_COPY_RVA, self.group_copy);
            set_marker_position(&BlitCall::new(0, self.marker.0, self.marker.1, 0, 0));
            for (n, id) in [(0, self.menu_ids.0), (1, self.menu_ids.1)] {
                if let Some(at) = menu_item_id(n) {
                    (at as *mut u32).write_unaligned(id);
                }
            }
        }
    }

    /// Which parts differ from `other`, for a failure message.
    fn differences(&self, other: &Self) -> String {
        let mut parts = Vec::new();
        for (name, same) in [
            ("line buffer", self.line == other.line),
            (
                "second heading",
                self.second_heading == other.second_heading,
            ),
            ("text buffer", self.text == other.text),
            ("scratch", self.scratch == other.scratch),
            ("finish", self.finish == other.finish),
            ("entry +0x34 copy", self.entry_34 == other.entry_34),
            ("group copy", self.group_copy == other.group_copy),
            ("marker", self.marker == other.marker),
            ("menu item ids", self.menu_ids == other.menu_ids),
        ] {
            if !same {
                parts.push(name);
            }
        }
        if parts.is_empty() {
            "nothing".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// The whole `dispatch` of a stage panel hook: `blank`, the Rust version when shadow is off,
/// and the shadow comparison. `rust` returns the carry flag.
pub fn dispatch_panel(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: &dyn Fn(&mut dyn Canvas) -> Result<bool, Refused>,
) {
    hook.count_call();
    if blanked(hook.name()) {
        return;
    }

    if !shadow() {
        match rust(&mut GameCanvas) {
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

    // Shadow: Rust goes through the motions without drawing; the original then does it all
    // again and really draws.
    let mut canvas = RecordingCanvas::new(true);
    let too_wide = match rust(&mut canvas) {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let left_r = unsafe { Leftovers::read() };

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let left_o = unsafe { Leftovers::read() };

    let ok = o.preserved_except(r, 0)
        && o.carry() == too_wide
        && left_o == left_r
        && unsafe { *state_ptr() } == canvas.state
        && (!recording_works() || calls == canvas.calls);
    if !ok {
        hook.mismatch(|| {
            format!(
                "original carry={} calls={} rust carry={too_wide} calls={} differs in: {}",
                o.carry(),
                calls.len(),
                canvas.calls.len(),
                left_o.differences(&left_r)
            )
        });
    }
    r.eflags = o.eflags;
}

/// Why the panel body cannot be tested right now, or None if it can: the hook that swallows
/// blitter calls, the string table, the fonts and the labels.
pub fn body_unavailable() -> Option<&'static str> {
    if !recording_works() {
        return Some("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let loaded = !blob().is_null()
        && FONTS.iter().all(|&f| font_is_loaded(f))
        && LABELS.iter().all(|&id| is_table_id(id));
    (!loaded).then_some("the string table or the fonts are not loaded")
}

/// Why the stage `record` cannot be used for a test, or None if it can. The original follows
/// its track pack entry's ids and indices without checks.
pub unsafe fn stage_unusable(record: u32) -> Option<&'static str> {
    unsafe {
        let Ok(entry) = record_entry(record as *const u8) else {
            return Some("the stage is not in the loaded track pack");
        };
        let description = dword(entry.wrapping_add(0x10));
        let ids = [
            dword(entry),
            dword(entry.wrapping_add(0x14)),
            description,
            description.wrapping_add(1),
            description.wrapping_add(2),
        ];
        if !ids.iter().all(|&id| is_table_id(id)) || dword(entry.wrapping_add(8)) >= 64 {
            return Some("the stage's track pack entry does not look like one");
        }
        None
    }
}

/// One self-test case for a stage panel hook: runs the original with the blitter calls
/// swallowed, restores what it wrote, runs `rust` the same way and compares the two. The
/// buffers, work variables and kerning table end up as they were.
pub fn check_panel(
    t: &mut SelfTest,
    call_original: CallOriginal,
    rust: &dyn Fn(&mut dyn Canvas) -> Result<bool, Refused>,
) {
    let saved = unsafe { Leftovers::read() };
    let saved_state = unsafe { *state_ptr() };

    let before = Regs::default();
    let mut ro = before;
    let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
    let left_o = unsafe { Leftovers::read() };
    let state_o = unsafe { *state_ptr() };

    unsafe { saved.restore() };
    let mut canvas = RecordingCanvas::new(false);
    let mine = rust(&mut canvas);
    let left_r = unsafe { Leftovers::read() };
    unsafe { saved.restore() };

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
            "original carry={} calls={} preserved={preserved}; rust {mine:?} calls={} first different call={first_difference:?} differs in: {}",
            ro.carry(),
            calls_o.len(),
            canvas.calls.len(),
            left_o.differences(&left_r)
        )
    });

    unsafe { *state_ptr() = saved_state };
}
