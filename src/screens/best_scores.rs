//! The best scores pages: scores pages 0 to 4 show a group of the best scores (ten entries a
//! group: nationality, car, name, score); page 5 shows a group of the class times (nationality,
//! car, time, name). The group and the caption string come from the page's entries in the group
//! and caption lists; the replays' range caption is here too. Drawing goes through
//! `game_screen::GameScreen`.

use crate::errors::Fault;
use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::menu::event_flow::StageTable;
use crate::menu::owner::Menus;
use crate::menu::pages_store::MenuPages;
use crate::records::stages_store::{Holder, RecordRef, RecordTable};
use crate::screens::game_screen::GameScreen;
use crate::screens::game_screen::{Held, HeldText};
use crate::system::owner::System;
use crate::text::draw_text::Canvas;
use rc2k_core::screens::best_scores::{Row, Rows};

/// The page's entry of one of the page title lists (`page_titles_a`, 12 entries, or
/// `page_titles_b`, 11). The page is 0 .. 10: only `menu::next_scores_page` (past 10 to 0) and
/// `menu::previous_scores_page` (below 0 to 10) write it, from 0. Refused past the list.
fn page_title(titles: &[PlainDword], pages: &MenuPages) -> Result<u32, BadInput> {
    titles
        .get(pages.read().scores_page as usize)
        .map(PlainDword::get)
        .ok_or(BadInput("a best scores page past its title"))
}

fn arcade_level(menus: &Menus) -> u32 {
    menus.play.arcade.read().level
}

fn event_class(menus: &Menus) -> u32 {
    menus.screens.event.shown.class.get()
}

fn event_stage_number(menus: &Menus) -> u32 {
    menus.screens.event.stage.number.get()
}

fn arcade_stage(menus: &Menus) -> u32 {
    menus.play.arcade.read().stage
}

/// Which typed table a page shows.
pub enum Source {
    /// The best scores (`BestTables::scores`): the name, the score, the
    /// highlight `new`.
    Scores,
    /// The levels' best scores (`BestTables::level_scores`): the name, the key,
    /// the highlight `mark`.
    LevelScores,
}

impl Source {
    fn need(
        &self,
        tables: &crate::saves::best_times::BestTables,
        what: &'static str,
    ) -> Result<(), BadInput> {
        let there = match self {
            Source::Scores => tables.scores.is_taken(),
            Source::LevelScores => tables.level_scores.is_taken(),
        };
        if there { Ok(()) } else { Err(BadInput(what)) }
    }
}

/// A scores page: its table and how it is drawn.
pub struct Table {
    pub at: Source,
    /// The best scores leave the last row's y in `DrawWork.button_x`.
    pub keeps_y: bool,
    /// Where the group's number is: a dword, or (None) the page's entry of the group list.
    pub group_at: Option<fn(&Menus) -> u32>,
    /// A fixed group instead.
    pub group_fixed: Option<u32>,
    /// An entry whose highlight field is 1 is drawn in the highlight fonts.
    pub highlight: bool,
    /// How far below the rank's y the sprites go (and `DrawWork.button_x` with them).
    pub sprite_dy: i32,
}

const BEST_SCORES: Table = Table {
    at: Source::Scores,
    keeps_y: true,
    group_at: None,
    group_fixed: None,
    highlight: false,
    sprite_dy: 0,
};
const CLASS_TIMES: Table = Table {
    at: Source::LevelScores,
    keeps_y: false,
    group_at: None,
    group_fixed: None,
    highlight: false,
    sprite_dy: 0,
};
/// `screens::draw_class_times_marked`'s: the class `arcade.level`, the player's
/// own times (`mark` 1) in the highlight fonts, the sprites 4 lower.
pub(super) const MARKED_CLASS_TIMES: Table = Table {
    at: Source::LevelScores,
    keeps_y: true,
    group_at: Some(arcade_level),
    group_fixed: None,
    highlight: true,
    sprite_dy: 4,
};

/// The family's screen for `rc2k_core::screens::best_scores`: the names are copied out
/// of the typed tables (`HeldText::Held`), the game's texts drawn where they are.
pub(super) type ScoresScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::best_scores::Font,
    rc2k_core::screens::best_scores::Work,
    HeldText,
>;

/// The page's group, from rank 1 until ten or an entry of -1: its name, its value, its rank (in
/// the rank font), the car's sprite and the flag's sprite (24 and 25 swapped) on the rank's
/// line; each text starts at the last one's y and the next row is 0x19 below. The sprites take
/// the line buffer past its NUL as their palette (unused). Refuses no table.
pub(super) fn draw_scores(
    o: crate::owners::Owners<'static>,
    t: &Table,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::best_scores::scores`.
    {
        t.at.need(&o.saves.best_tables, "no best scores table")?;
        let group = match (t.group_fixed, t.group_at) {
            (Some(g), _) => g,
            (None, Some(a)) => a(menus),
            (None, None) => page_title(&menus.lists.page_titles_b, &menus.tables.pages)?,
        };
        let ten = match t.at {
            Source::Scores => o
                .saves
                .best_tables
                .scores
                .with(|s| s.groups.get(group as usize).copied())?
                .ok_or(BadInput("a group past the best scores' five"))?
                .map(|e| HeldRow {
                    name: e.name_text(),
                    figure: e.score,
                    nation: e.nation,
                    car: e.car,
                    marked: e.new == 1,
                    ends: e.name_id == rc2k_core::saves::name_id::NameId::LIST_END,
                }),
            Source::LevelScores => o
                .saves
                .best_tables
                .level_scores
                .with(|s| s.levels.get(group as usize).copied())?
                .ok_or(BadInput("a group past the levels' seven"))?
                .map(|e| HeldRow {
                    name: e.name_text(),
                    figure: e.key,
                    nation: e.nation,
                    car: e.car,
                    marked: e.mark == 1,
                    ends: e.first == u32::MAX,
                }),
        };
        draw_held_rows(o, t, &ten, canvas)
    }
}

/// A row of a typed table, copied out of it.
struct HeldRow {
    /// None: no NUL before the entry's end.
    name: Option<Vec<u8>>,
    figure: u32,
    nation: u32,
    car: u32,
    /// The entry's highlight field is 1 (drawn so only with `t.highlight`).
    marked: bool,
    /// The list ends here (its first dword -1).
    ends: bool,
}

/// Row `e` for the core: a name without its end is refused after the drawing (through
/// `fault`; drawn empty meanwhile).
fn held_row(fault: &Fault, e: &HeldRow, highlight: bool) -> Row<HeldText> {
    let name = fault.ok(e
        .name
        .as_deref()
        .and_then(Held::new)
        .map(Some)
        .ok_or(BadInput("a best entry's name without its end")));
    Row {
        name: HeldText::Held(name.unwrap_or(Held::EMPTY)),
        figure: e.figure,
        nation: e.nation,
        car: e.car,
        marked: highlight && e.marked,
    }
}

/// `draw_scores` on ten rows of a typed table.
fn draw_held_rows(
    o: crate::owners::Owners<'static>,
    t: &Table,
    ten: &[HeldRow; 10],
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let fault = Fault::default();
    let row = |n: u32| held_row(&fault, &ten[n as usize], t.highlight);
    let ends = |n: u32| ten[n as usize].ends;
    let rows = Rows {
        row: &row,
        ends_at: &ends,
    };
    let mut screen = ScoresScreen::new(canvas, o);
    let drawn = rc2k_core::screens::best_scores::scores(&mut screen, &rows, t.keeps_y, t.sprite_dy);
    fault.check()?;
    drawn
}

/// The best scores for pages up to 4, else the class times.
pub fn draw_best_scores_page(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let pages = &o.menus.tables.pages;
    if (pages.read().scores_page as i32) <= 4 {
        draw_scores(o, &BEST_SCORES, canvas)
    } else {
        draw_scores(o, &CLASS_TIMES, canvas)
    }
}

/// The caption string from the page's entry of the caption list and its NUL.
pub fn best_scores_caption(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    lists: &crate::menu::lists_store::MenuLists,
    pages: &MenuPages,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::captions::fit` (through `car_caption::put_string`);
    // refused when the caption would pass 0x100 bytes.
    let id = page_title(&lists.page_titles_a, pages)?;
    crate::screens::car_caption::put_string(so, id, &text.caption, 0x100)
}

/// The caption "n-m", n = `pages.replay_page` x 20, m = n + 19 (`strings::append_u32`,
/// `append_minus`).
pub fn replay_range_caption(
    pages: &MenuPages,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), BadInput> {
    // The logic is in `rc2k_core::screens::best_scores::range_caption`.
    let mut bytes = rc2k_core::screens::best_scores::range_caption(pages.read().replay_page);
    bytes.push(0);
    text.caption.write_at(0, &bytes, "the caption")
}

/// A stage times page: where its stage record is and which table holds its times.
pub struct StageTimes {
    /// The stage record's table (the records 0xA4 bytes, each with the stage's name string and
    /// its times' number) and the record's number.
    pub records: fn(&Menus, &System) -> Result<RecordTable, BadInput>,
    pub record: fn(&Menus) -> u32,
    /// The times' table (ten for each stage; the record names the stage's tables).
    pub table: for<'t> fn(&Menus, &'t crate::saves::best_times::BestTables) -> &'t StageTable,
    /// The rally page leaves the sprites' y in `DrawWork.button_x` and the flag's glyph in
    /// `DrawWork.button_y`.
    pub rally: bool,
}

fn champ_records(menus: &Menus, system: &System) -> Result<RecordTable, BadInput> {
    let screens = &menus.screens;
    system.records().stage_tables.group_event_table(
        screens.event.stage.group.get(),
        screens.event.stage.sub.get(),
    )
}

fn champ_table<'t>(
    menus: &Menus,
    tables: &'t crate::saves::best_times::BestTables,
) -> &'t StageTable {
    if menus.screens.event.stage.group.get() == 1 {
        &tables.stage_times_alt
    } else {
        &tables.time_trial_times
    }
}

fn rally_records(menus: &Menus, system: &System) -> Result<RecordTable, BadInput> {
    system
        .records()
        .stage_tables
        .level_table(menus.play.arcade.read().level)
}

fn rally_table<'t>(_: &Menus, tables: &'t crate::saves::best_times::BestTables) -> &'t StageTable {
    &tables.course_times
}

/// `screens::draw_stage_times`'.
pub(super) const STAGE_TIMES: StageTimes = StageTimes {
    records: champ_records,
    record: event_stage_number,
    table: champ_table,
    rally: false,
};
/// `screens::draw_rally_stage_times`'.
pub(super) const RALLY_STAGE_TIMES: StageTimes = StageTimes {
    records: rally_records,
    record: arcade_stage,
    table: rally_table,
    rally: true,
};

/// The stage's name (the string of the records' name field) and its stage line (string 0x482 ..., built
/// in the line buffer), in the heading font. Used for the stage and rally times headings.
/// Refuses no stage records.
pub(super) fn draw_times_heading(
    o: crate::owners::Owners<'static>,
    system: &System,
    p: &StageTimes,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::best_scores::heading`.
    {
        let base = (p.records)(menus, system)?;
        let id = base.title(&system.records().stage_tables)?;
        let record = base.row(&system.records().stage_tables, (p.record)(menus))?;
        let line = || -> Result<Vec<u8>, Refused> {
            stage_line_without_nul(
                crate::strings::append_string_by_id::StringOwners::of(o),
                0x482,
                record,
            )
        };
        rc2k_core::screens::best_scores::heading(&mut ScoresScreen::new(canvas, o), id, Some(&line))
    }
}

/// The stage line (`page_settings::stage_line_bytes`) without its NUL; refused past the line
/// buffer.
fn stage_line_without_nul(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    id: u32,
    record: RecordRef,
) -> Result<Vec<u8>, Refused> {
    let mut line = crate::screens::page_settings::stage_line_bytes(so, id, record)?;
    if line.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    line.pop();
    Ok(line)
}

/// For `screens::draw_stage_times` / `screens::draw_rally_stage_times`: the heading, then up to ten
/// times of the stage's group (the record's table number, its stage in the table): name (x 0xF0), time
/// (`strings::append_time`, drawn in fixed columns ending at x 0x276), rank (x 0xC8), the car's and
/// flag's sprites; the player's own in the highlight fonts; rows 0x19 apart from the last text's y.
/// Refuses (before anything) no stage records or no times table.
pub fn draw_stage_times(
    o: crate::owners::Owners<'static>,
    system: &System,
    p: &StageTimes,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let base = (p.records)(menus, system)?;
    let table = (p.table)(menus, &o.saves.best_tables);
    if !table.is_taken() {
        return Err(BadInput("no times table").into());
    }
    draw_times_heading(o, system, p, canvas)?;
    let group = base
        .row(&system.records().stage_tables, (p.record)(menus))?
        .record(&system.records().stage_tables)?
        .event_slot();
    let keep = if p.rally { Keep::Below4 } else { Keep::Nothing };
    draw_time_rows(o, table, group as usize, keep, canvas)?;
    Ok(())
}

pub use rc2k_core::screens::best_scores::Keep;

/// Up to ten times rows of stage `stage` of `table` (the player's own, `mark` 1,
/// highlighted). Gives whether all ten were drawn (the list did not end early: the entry
/// after the ten, the next stage's first or the block's zeros past the table, is read for
/// it). Refuses a stage past the table, and an entry after the ten past the block.
/// The logic is in `rc2k_core::screens::best_scores::time_rows`.
fn draw_time_rows(
    o: crate::owners::Owners<'static>,
    table: &StageTable,
    stage: usize,
    keep: Keep,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let (ten, after) = table.with(|t| {
        (
            t.stages.get(stage).copied(),
            t.first_at((stage + 1) * rc2k_core::saves::entries::PER_LIST),
        )
    })?;
    let ten = ten
        .ok_or(BadInput("a stage past its best times table"))?
        .map(|e| HeldRow {
            name: e.name_text(),
            figure: e.head.key,
            nation: e.head.nation,
            car: e.head.car,
            marked: e.head.mark == 1,
            ends: e.head.first == u32::MAX,
        });
    let fault = Fault::default();
    let row = |n: u32| held_row(&fault, &ten[n as usize], true);
    let ends = |n: u32| match ten.get(n as usize) {
        Some(e) => e.ends,
        None => fault.ok(after.map(|v| v == u32::MAX).ok_or(BadInput(
            "an entry after the ten past the best times' block",
        ))),
    };
    let rows = Rows {
        row: &row,
        ends_at: &ends,
    };
    let show_hours = crate::strings::append_time::show_hours(o.strings);
    let drawn = rc2k_core::screens::best_scores::time_rows(
        &mut ScoresScreen::new(canvas, o),
        &rows,
        keep,
        show_hours,
    );
    fault.check()?;
    drawn
}

/// `records::find_record` of the record table: the section's record `index`.
pub fn record_at(
    menus: &Menus,
    system: &System,
    index: u32,
) -> Result<RecordRef, crate::errors::BadInput> {
    crate::records::find_record::find_record_in(
        &system.records().stage_tables,
        system.records().stage_tables.championship(),
        index as i32,
        menus.screens.event.section.get(),
    )
}

/// Whether the record table has the section (the look-up has no end): for the refusals.
pub(super) fn section_there(
    menus: &Menus,
    system: &System,
) -> Result<bool, crate::errors::BadInput> {
    crate::records::find_record::has_section(
        &system.records().stage_tables,
        system.records().stage_tables.championship(),
        menus.screens.event.section.get(),
        0x400,
    )
}

/// The section's heading at (0x2EB, 0x77) in the heading font (in event mode 0x13 the section's
/// entry of the heading list, else the section's first record's name string), then record
/// `index`'s stage line (string 0x481 ...) in the line buffer at (0x2EB, 0x91). Refuses (before
/// anything) a section the table has not.
pub fn draw_record_heading(
    o: crate::owners::Owners<'static>,
    system: &System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::best_scores::heading`.
    {
        if !section_there(menus, system)? {
            return Err(BadInput("a section past the record table").into());
        }
        let first = record_at(menus, system, 0)?;
        let id = if menus.play.champ_view.event_mode.get() == 0x13 {
            crate::screens::stage_panel::section_title(
                &menus.lists,
                menus.screens.event.section.get(),
            )?
        } else {
            first.record(&system.records().stage_tables)?.name()
        };
        let record = record_at(menus, system, menus.screens.event.stage_in_section.get())?;
        let line = || -> Result<Vec<u8>, Refused> {
            stage_line_without_nul(
                crate::strings::append_string_by_id::StringOwners::of(o),
                0x481,
                record,
            )
        };
        rc2k_core::screens::best_scores::heading(&mut ScoresScreen::new(canvas, o), id, Some(&line))
    }
}

/// The record's heading, its best times (the rows' y left in `DrawWork.button_x`), and the
/// fastest line (string 0x1A2, the car's sprite, the name, the flag's sprite, the speed with its
/// unit), also when the list ended before ten rows. Refuses a
/// section the table lacks, no times table and what `strings::append_speed` refuses.
pub fn draw_record_times(
    o: crate::owners::Owners<'static>,
    system: &System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let times = &o.saves.best_tables.stage_times;
    let (true, true) = (section_there(menus, system)?, times.is_taken()) else {
        return Err(BadInput("a section past the record table, or no times table").into());
    };
    let record = record_at(menus, system, menus.screens.event.stage_in_section.get())?;
    // The record's fields are read at once (nothing drawn in between writes them).
    let fastest = record
        .record(&system.records().stage_tables)?
        .holder(Holder::Fastest);
    let (top_speed, times_at, nation, car) = (
        fastest.speed,
        record.record(&system.records().stage_tables)?.event_slot(),
        fastest.nation,
        fastest.car,
    );
    let speed = crate::strings::append_speed::append_speed(
        crate::strings::append_string_by_id::StringOwners::of(o),
        top_speed,
        true,
        crate::strings::units::units(o.strings),
    )?;
    draw_record_heading(o, system, canvas)?;
    draw_time_rows(o, times, times_at as usize, Keep::Y, canvas)?;
    rc2k_core::screens::best_scores::fastest(
        &mut ScoresScreen::new(canvas, o),
        nation,
        HeldText::Held(record.holder_text(&system.records().stage_tables, Holder::Fastest)?),
        car,
        &speed,
    )
}

/// `screens::draw_event_best_scores`': group `event.shown.class`, the player's own
/// (its player flag 1) in the highlight fonts.
pub(super) const EVENT_BEST_SCORES: Table = Table {
    group_at: Some(event_class),
    highlight: true,
    ..BEST_SCORES
};
/// `screens::draw_overall_best_scores`': group 4.
pub(super) const OVERALL_BEST_SCORES: Table = Table {
    group_fixed: Some(4),
    highlight: true,
    ..BEST_SCORES
};

/// The heading in the heading font: in event mode 0x13 the section's name (the section's entry
/// of the heading list), else string `other` (the first: 0x35E + `event.shown.class`; the second
/// 0x362); then the scores as `draw_scores`. Refuses no table.
pub(super) fn draw_scores_with_heading(
    o: crate::owners::Owners<'static>,
    t: &Table,
    event: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    t.at.need(&o.saves.best_tables, "no best scores table")?;
    // The logic is in `rc2k_core::screens::best_scores::scores_heading`, `heading`.
    let section = (menus.play.champ_view.event_mode.get() == 0x13)
        .then(|| {
            crate::screens::stage_panel::section_title(
                &menus.lists,
                menus.screens.event.section.get(),
            )
        })
        .transpose()?;
    let mut screen = ScoresScreen::new(canvas, o);
    let id = rc2k_core::screens::best_scores::scores_heading(
        section,
        event.then(|| menus.screens.event.shown.class.get()),
    );
    rc2k_core::screens::best_scores::heading(&mut screen, id, None)?;
    draw_scores(o, t, screen.canvas)
}
