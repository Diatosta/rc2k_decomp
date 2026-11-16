//! What the four stage panel functions share: `draw_stage_info`, `draw_service_stage_info`,
//! `draw_time_trial_stage_info` and `draw_selected_stage_menu`. They use the same layout and differ
//! only in where the stage comes from, what goes above the panel and which best-time table the
//! record rows use.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_marker_pair::marker_calls;
use crate::gfx::owner::Gfx;
use crate::memory::session::{DwordLike, PlainDword};
use crate::menu::owner::Screens;
use crate::records::stages_store::RecordRef;
use crate::records::stages_store::TrackField;
use crate::records::trakpak_entry::track_of;
use crate::screens::draw_stage_heading::heading_text;
use crate::screens::draw_stage_records::draw_stage_records;
use crate::screens::game_screen::GameScreen;
use crate::strings::units::units;
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;

// ---- game data ----

// mmss: receives the finish time
// the summary copy[1]: copy of entry +0x34
const SECOND_HEADING_STRING: u32 = 0x482;

pub const TITLE_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::STAGE_TITLE;
const HEADING_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING;
/// The title string of stage group `group` (`Results.group_titles`). The group is 0 or 1:
/// only `menu::switch_stage_group` (0 <-> 1) and `menu::view_courses_loop` (0) write it,
/// and no saved file holds it. Refused past the field.
pub fn group_title(group: u32) -> Result<u32, BadInput> {
    crate::menu::results_store::Results::group_title(group)
}

/// The title string of section `section` of the A8 championship (`MenuLists.section_titles`,
/// six titles and a -1). The section is one the championship's record table has (five section
/// records, so 0 .. 5): the championships start it at 0 or at their start setting less one and
/// add one only after a section's last record, `menu::view_stage_loop` puts back what it
/// took, and a savegame holds what the game saved; a section the table has not is refused by
/// the record look-ups before any title is drawn. Refused past the field.
pub fn section_title(
    lists: &crate::menu::lists_store::MenuLists,
    section: u32,
) -> Result<u32, BadInput> {
    lists
        .section_titles
        .get(section as usize)
        .map(PlainDword::get)
        .ok_or(BadInput("a section past its title"))
}

// ---- the shared parts ----

/// Draws a string id with one of the game's font objects.
pub fn label(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    canvas: &mut dyn Canvas,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: i32,
    y: i32,
) -> Result<(), Refused> {
    { draw_string_by_id_at(so, id, font, x, y, canvas) }.map(|_| ())
}

/// Builds the second heading ("<string 0x482> n/6: name") in its own buffer. Three of the
/// four panels build it; only `draw_time_trial_stage_info` also draws it (the text it
/// returns, with its NUL).
pub fn build_second_heading(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    record: RecordRef,
) -> Result<Vec<u8>, Refused> {
    let bytes = heading_text(so, record, SECOND_HEADING_STRING)?;
    so.menus
        .text_buffers
        .stage_heading
        .write_at(0, &bytes, "the second stage heading")?;
    Ok(bytes)
}

/// Everything from the map markers down to the numbered line at the bottom left, for the
/// stage `record`. `marker_palette` is whatever the caller passes for the markers (unused).
/// `keep_entry_34` is the one store that two of the panels add: the entry's field at +0x34 into
/// RaceState.settings.rivals[1].
///
/// The stage's track pack entry holds most of what is shown: +0x08 index of its duration,
/// +0x0C length in hundredths of a mile (0 = none), +0x10 first of three description strings,
/// +0x14 weather string, +0x1C temperature, +0x20 wind in tenths of a mile per hour,
/// +0x24/+0x28/+0x2C/+0x30 surface percentages, +0x38 a number and +0x3C which of two labels
/// goes with it. The record gives +0x10, the start time as hours * 100 + minutes.
pub fn draw_stage_body(
    o: crate::owners::Owners<'static>,
    gfx: &Gfx,
    record: RecordRef,
    keep_entry_34: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    draw_stage_body_in(o, gfx, record, keep_entry_34, HEADING_FONT, canvas)
}

/// `draw_stage_body` with the two headings (surface, weather) in `heading_font`: the rally
/// stage page (`screens::draw_rally_stage_info`) uses `FontObjects.objects[14].first_char`, the
/// others `FontObjects.objects[17].first_char`.
pub fn draw_stage_body_in(
    o: crate::owners::Owners<'static>,
    gfx: &Gfx,
    record: RecordRef,
    keep_entry_34: bool,
    heading_font: crate::gfx::typefaces::FontId,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::stage_panel::body`.
    use rc2k_core::screens::stage_panel::{Entry, Font};
    // The two markers on the map.
    for call in marker_calls(gfx, menus, record.record(&o.system.records().stage_tables)?) {
        canvas.blit_glyph(call)?;
    }
    let entry = || -> Result<Entry, Refused> {
        // By name: past the pack's 36 entries the number reads on into what follows it
        // (`trakpak_entry::Track::get`), and every field here may be reached so.
        let stages = &o.system.records().stage_tables;
        let track = track_of(stages, record)?;
        let fault = crate::menu::event_flow::Fault::default();
        let f = |field: TrackField| fault.ok(track.get(stages, field));
        let made = Entry {
            // `MenuLists::durations` (the stage durations) has 37 entries; a duration index past it
            // is refused.
            duration: menus
                .lists
                .durations
                .get(f(TrackField::Duration) as usize)
                .ok_or(BadInput("a stage duration past the 37"))?
                .get(),
            length: f(TrackField::Length),
            description: f(TrackField::Description),
            weather: f(TrackField::Weather),
            temperature: f(TrackField::Temperature) as i32,
            wind: f(TrackField::Wind),
            surfaces: TrackField::SURFACES.map(&f),
            stage_number: f(TrackField::Stage),
            number: f(TrackField::Number),
            number_label_one: f(TrackField::NumberLabelOne) == 1,
        };
        fault.check()?;
        Ok(made)
    };
    let heading = if heading_font == HEADING_FONT {
        Font::Heading
    } else {
        Font::RallyHeading
    };
    let mut screen = PanelScreen::new(canvas, o);
    rc2k_core::screens::stage_panel::body(
        &mut screen,
        record.record(&o.system.records().stage_tables)?.hhmm(),
        &entry,
        units(o.strings),
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

/// Stage `stage`'s best time (its first) in `table`, copied out. Refuses no table and a stage
/// past it.
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
/// `table`. Returns whether the last text was too wide for its box.
pub fn draw_records_from(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    table: &crate::menu::event_flow::StageTable,
    record: RecordRef,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let best = best_of(
        table,
        record.record(&system.records().stage_tables)?.event_slot(),
    )?;
    draw_stage_records(o, system, &best, canvas)
}

/// The record rows as the two "selected stage" panels draw them: from the group 1 table when
/// group 1 is selected, else from the usual one, and not at all while either the group 1
/// table or the chosen one is missing. Returns whether the last text was too wide
/// for its box (false when nothing is drawn).
pub fn draw_records_for_group(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    group: u32,
    record: RecordRef,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let tables = &o.saves.best_tables;
    if !tables.course_times.is_taken() {
        return Ok(false);
    }
    let table = if group != 1 {
        &tables.stage_times
    } else {
        &tables.course_times
    };
    if !table.is_taken() {
        return Ok(false);
    }
    draw_records_from(o, system, table, record, canvas)
}

// ---- support ----

/// The string id (+0x14) of item `n` (from 0) of the current menu, or None without a menu
/// (or past its end).
pub fn menu_item_id<'a>(
    screens: &'a Screens,
    tables: &'a crate::menu::menus_store::MenuTables,
    n: u32,
) -> Option<&'a dyn DwordLike> {
    let menu = screens.loop_flags.menu.get().ok()?;
    menu.items(tables)
        .get(n as usize)
        .map(|i| &i.string as &dyn DwordLike)
}
