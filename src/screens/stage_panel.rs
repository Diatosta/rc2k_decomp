//! What the four stage panel functions share: `draw_stage_info`, `draw_service_stage_info`,
//! `draw_time_trial_stage_info` and `draw_selected_stage_menu`. In the original each is a full copy of
//! the same layout; they differ only in where the stage comes from, what goes above the panel
//! and which best-time table the record rows use.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_marker_pair::{marker_calls, set_marker_position};
use crate::gfx::owner::Gfx;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::race::owner::Race;
use crate::records::stages_store::RecordRef;
use crate::records::trakpak_entry::record_entry;
use crate::screens::draw_stage_heading::heading_text;
use crate::screens::draw_stage_records::draw_stage_records;
use crate::screens::game_screen::GameScreen;
use crate::session::DwordLike;
use crate::strings::units::units;
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;

// ---- game data ----

// `draw_work.figure`: a work variable used all over the game
// mmss: receives the finish time
// the summary copy[1]: copy of entry +0x34
const SECOND_HEADING_STRING: u32 = 0x482;

pub const TITLE_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::STAGE_TITLE;
const HEADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING;
/// The dword `index` of `table`, reading on only inside `holder`, the store holding it.
pub fn table_dword_at<H: crate::cells::Cells>(
    holder: &H,
    table: &[impl crate::session::Cell32],
    index: u32,
) -> Result<u32, crate::errors::BadInput> {
    crate::session::read_on_in(holder, table, index as usize, "a table's dword")
}

// ---- the shared parts ----

/// Draws a string id with one of the game's font objects.
pub fn label(
    canvas: &mut dyn Canvas,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: i32,
    y: i32,
) -> Result<(), Refused> {
    { draw_string_by_id_at(id, font, x, y, canvas) }.map(|_| ())
}

/// Builds the second heading ("<string 0x482> n/6: name") in its own buffer. Three of the
/// four panels build it; only `draw_time_trial_stage_info` also draws it (the text it
/// returns, with its NUL).
pub fn build_second_heading(race: &Race, record: RecordRef) -> Result<Vec<u8>, Refused> {
    let bytes = heading_text(record, SECOND_HEADING_STRING)?;
    race.text_buffers()
        .stage_heading
        .write_at(0, &bytes, "the second stage heading")?;
    Ok(bytes)
}

/// Everything from the map markers down to the numbered line at the bottom left, for the
/// stage `record`. `marker_palette` is whatever the caller has in ebx when the markers are
/// drawn. `keep_entry_34` is the one store that two of the panels add: entry +0x34 into
/// RaceState.settings.rivals[1].
///
/// The stage's track pack entry holds most of what is shown: +0x08 index of its duration,
/// +0x0C length in hundredths of a mile (0 = none), +0x10 first of three description strings,
/// +0x14 weather string, +0x1C temperature, +0x20 wind in tenths of a mile per hour,
/// +0x24/+0x28/+0x2C/+0x30 surface percentages, +0x38 a number and +0x3C which of two labels
/// goes with it. The record gives +0x10, the start time as hours * 100 + minutes.
pub fn draw_stage_body(
    menus: &Menus,
    gfx: &Gfx,
    record: RecordRef,
    keep_entry_34: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    draw_stage_body_in(menus, gfx, record, keep_entry_34, HEADING_FONT, canvas)
}

/// `draw_stage_body` with the two headings (surface, weather) in `heading_font`: the rally
/// stage page (`screens::draw_rally_stage_info`) uses `FontObjects.objects[14].first_char`, the others
/// `FontObjects.objects[17].first_char`.
pub fn draw_stage_body_in(
    menus: &Menus,
    gfx: &Gfx,
    record: RecordRef,
    keep_entry_34: bool,
    heading_font: crate::gfx::typefaces::FontId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::stage_panel::body`.
    use rc2k_core::screens::stage_panel::{Entry, Font};
    // The two markers on the map.
    for call in marker_calls(gfx, menus, record.record()?) {
        set_marker_position(menus, &call);
        canvas.blit_glyph(call)?;
    }
    let entry = || -> Result<Entry, Refused> {
        // By offset: past the pack's 36 entries the original's index reads on into what
        // follows it (`trakpak_entry::Track`), and every field here may be reached so.
        let entry = record_entry(record)?;
        let fault = crate::menu::event_flow::Fault::default();
        let f = |o: u32| fault.ok(crate::records::trakpak_entry::run_on(entry + o as usize));
        let made = Entry {
            // `MENU_LISTS.durations` (the stage durations) has 37 entries; the original's index
            // has no bound (fixed: refused).
            duration: menus
                .tables
                .lists
                .durations
                .get(f(8) as usize)
                .ok_or(BadInput("a stage duration past the 37"))?
                .get(),
            length: f(0x0C),
            description: f(0x10),
            weather: f(0x14),
            temperature: f(0x1C) as i32,
            wind: f(0x20),
            surfaces: [f(0x24), f(0x28), f(0x2C), f(0x30)],
            stage_number: f(0x34),
            number: f(0x38),
            number_label_one: f(0x3C) == 1,
        };
        fault.check()?;
        Ok(made)
    };
    let heading = if heading_font == HEADING_FONT {
        Font::Heading
    } else {
        Font::RallyHeading
    };
    let mut screen = PanelScreen::new(canvas);
    rc2k_core::screens::stage_panel::body(
        &mut screen,
        record.record()?.hhmm(),
        &entry,
        units(),
        heading,
        keep_entry_34,
    )
}

/// The panel's screen for `rc2k_core::screens::stage_panel`: the best times' names copied
/// out of their tables (`HeldText::Held`), the game's texts drawn where they are.
pub type PanelScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::stage_panel::Font,
    rc2k_core::screens::stage_panel::Work,
    crate::screens::game_screen::HeldText,
>;

/// Stage `stage`'s best time (its first) in `table`, copied out. Refuses no table (the
/// original reads near 0) and a stage past it.
pub fn best_of(
    table: &crate::menu::event_flow::StageTable,
    stage: u32,
) -> Result<rc2k_core::saves::entries::BestTime, crate::errors::BadInput> {
    if !table.is_taken() {
        return Err(crate::errors::BadInput("no best times table"));
    }
    table
        .with(|t| t.stages.get(stage as usize).map(|s| s[0]))?
        .ok_or(crate::errors::BadInput("a stage past its best times table"))
}

/// The record rows, with the stage's best time (the record's +0x3C its stage) from
/// `table`. Returns the carry flag.
pub fn draw_records_from(
    menus: &Menus,
    system: &crate::system::owner::System,
    table: &crate::menu::event_flow::StageTable,
    record: RecordRef,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let best = best_of(table, record.record()?.event_slot())?;
    draw_stage_records(menus, system, &best, canvas)
}

/// The record rows as the two "selected stage" panels draw them: from the group 1 table when
/// group 1 is selected, else from the usual one, and not at all while either the group 1
/// table or the chosen one is missing. Returns the carry flag (clear when nothing is drawn).
pub fn draw_records_for_group(
    menus: &Menus,
    system: &crate::system::owner::System,
    group: u32,
    record: RecordRef,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    use crate::saves::best_times::{COURSE_TIMES, STAGE_TIMES};
    if !COURSE_TIMES.is_taken() {
        return Ok(false);
    }
    let table = if group != 1 {
        &STAGE_TIMES
    } else {
        &COURSE_TIMES
    };
    if !table.is_taken() {
        return Ok(false);
    }
    draw_records_from(menus, system, table, record, canvas)
}

// ---- support ----

/// The string id (+0x14) of item `n` (from 0) of the current menu, or None without a menu
/// (or past its end).
pub fn menu_item_id(screens: &Screens, n: u32) -> Option<&'_ dyn DwordLike> {
    let menu = screens.loop_flags.menu.get().ok()?;
    menu.items()
        .get(n as usize)
        .map(|i| &i.string as &dyn DwordLike)
}
