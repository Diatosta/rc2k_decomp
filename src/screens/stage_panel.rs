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
use crate::records::trakpak_entry::record_entry;
use crate::screens::draw_stage_heading::heading_text;
use crate::screens::draw_stage_records::draw_stage_records;
use crate::screens::game_screen::GameScreen;
use crate::screens::{blanked, font_at, font_is_loaded, line_buffer, set_line};
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::strings::units::units;
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
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

const START_LABEL: u32 = 0x34D;
const FINISH_LABEL: u32 = 0x34E;
const TOTAL_LABEL: u32 = 0x356;
const LENGTH_LABEL: u32 = 0x34F;
const LENGTH_UNIT_IMPERIAL: u32 = 0x56F;
const LENGTH_UNIT_METRIC: u32 = 0x570;
const SURFACE_HEADING: u32 = 0x345;
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
    // Converted: `rc2k_core::screens::stage_panel::body`.
    use rc2k_core::screens::stage_panel::{Entry, Font};
    unsafe {
        // The two markers on the map.
        for call in marker_calls(record as *const u8, marker_palette, sprite_table()) {
            set_marker_position(&call);
            canvas.blit_glyph(call)?;
        }
        let entry = || -> Result<Entry, Refused> {
            let entry = record_entry(record as *const u8)?;
            let f = |o: u32| dword(entry.wrapping_add(o));
            Ok(Entry {
                duration: table_dword(DURATIONS_RVA, f(8)),
                length: f(0x0C),
                description: f(0x10),
                weather: f(0x14),
                temperature: f(0x1C) as i32,
                wind: f(0x20),
                surfaces: [f(0x24), f(0x28), f(0x2C), f(0x30)],
                entry_34: f(0x34),
                number: f(0x38),
                number_label_one: f(0x3C) == 1,
            })
        };
        let heading = if heading_font == HEADING_FONT {
            Font::Heading
        } else {
            Font::RallyHeading
        };
        let mut screen = PanelScreen::new(canvas);
        rc2k_core::screens::stage_panel::body(
            &mut screen,
            dword(record.wrapping_add(0x10)),
            &entry,
            units(),
            heading,
            keep_entry_34,
        )
    }
}

/// The panel's screen for `rc2k_core::screens::stage_panel`.
pub type PanelScreen<'a> =
    GameScreen<'a, rc2k_core::screens::stage_panel::Font, rc2k_core::screens::stage_panel::Work>;

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
