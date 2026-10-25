//! What the four stage panel functions share: `draw_stage_info`, `draw_service_stage_info`,
//! `draw_time_trial_stage_info` and `draw_selected_stage_menu`. In the original each is a full copy of
//! the same layout; they differ only in where the stage comes from, what goes above the panel
//! and which best-time table the record rows use.

use crate::errors::{FatalError, Refused};
use crate::gfx::blit_marker_pair::{marker_calls, set_marker_position, sprite_table};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::records::trakpak_entry::record_entry;
use crate::screens::draw_stage_heading::heading_text;
use crate::screens::draw_stage_records::draw_stage_records;
use crate::screens::game_screen::GameScreen;
use crate::strings::units::units;
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;
use std::ptr::copy_nonoverlapping;

// ---- game data ----

// dword_7559EC: a work variable used all over the game
// mmss: receives the finish time
// dword_75ACD0[1]: copy of entry +0x34
pub const BEST_TIME_ENTRY_SIZE: u32 = 0x618;
const SECOND_HEADING_STRING: u32 = 0x482;

pub const TITLE_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::STAGE_TITLE;
const HEADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING;
pub unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The dword `index` of the table at `table` (an address; no bounds, as the original).
pub unsafe fn table_dword_at(table: usize, index: u32) -> u32 {
    unsafe { dword((table as u32).wrapping_add(index.wrapping_mul(4))) }
}

// ---- the shared parts ----

/// Draws a string id with one of the game's font objects.
pub unsafe fn label(
    canvas: &mut dyn Canvas,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: i32,
    y: i32,
) -> Result<(), Refused> {
    unsafe { draw_string_by_id_at(id, font.as_ptr(), x, y, canvas) }.map(|_| ())
}

/// Builds the second heading ("<string 0x482> n/6: name") in its own buffer. Three of the
/// four panels build it; only `draw_time_trial_stage_info` also draws it.
pub unsafe fn build_second_heading(record: u32) -> Result<(), FatalError> {
    unsafe {
        let bytes = heading_text(record as *const u8, SECOND_HEADING_STRING)?;
        copy_nonoverlapping(
            bytes.as_ptr(),
            crate::screens::text_buffers_store::TEXT_BUFFERS
                .stage_heading
                .as_ptr(),
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
    heading_font: crate::gfx::typefaces::FontId,
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
                // `MENU_LISTS.durations` (dword_75A49A) has 37 entries; the original's index
                // has no bound: each address resolved to its home by `MOVED`.
                duration: crate::store::game_read::<u32>(crate::framework::rva(
                    0x75A49A_usize.wrapping_add((f(8) as usize).wrapping_mul(4)),
                )),
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
        let mut table: u32 = CHAMP_VIEW.course_best_times.get();
        if table == 0 {
            return Ok(false);
        }
        if group != 1 {
            table = CHAMP_VIEW.stage_best_times.get();
        }
        if table == 0 {
            return Ok(false);
        }
        draw_records_from(table, record, canvas)
    }
}

// ---- support ----

/// Address of the string id of item `n` of the current menu, or None without a menu.
pub unsafe fn menu_item_id(n: u32) -> Option<u32> {
    let menu: u32 = { LOOP_FLAGS.menu.get() };
    (menu != 0).then(|| menu.wrapping_add(4 + n * 0x5C + 0x14))
}
