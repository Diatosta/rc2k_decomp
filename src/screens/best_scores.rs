//! The best scores pages (`menu::view_best_scores_loop`'s lists; stream A, part F1,
//! 2026-10-18): page `dword_755240` 0 .. 4 shows a group of the best scores
//! (`dword_7556D4`: groups of ten 0x3C-byte entries, 0x258 a group; +8 the nationality, +0xC
//! the car, +0x18 the name, +0x38 the score), 5 on a group of the class times (`dword_7556DC`:
//! groups of ten 0x50-byte entries, 0x320 a group; +8 the nationality, +0x10 the car, +0x14
//! the time, +0x20 the name); the group by `dword_75A2D6[page]`, the caption string by
//! `dword_75A2A6[page]`. And the replays' range caption. Converted 2026-10-20:
//! `rc2k_core::screens::best_scores` draws through `game_screen::GameScreen`.

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::Fault;
use crate::menu::event_flow::StageTable;
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use crate::records::stages_store::{Holder, RecordRef, RecordTable};
use crate::screens::game_screen::GameScreen;
use crate::screens::game_screen::{Held, HeldText};
use crate::session::Dword;
use crate::system::owner::system;
use crate::text::draw_text::Canvas;
use rc2k_core::screens::best_scores::{Row, Rows};

const PAGE: &Dword = &crate::menu::owner::MENUS.tables.pages.scores_page; // dword_755240
/// Which typed table a page shows.
pub enum Source {
    /// The best scores (`saves::best_times::SCORES`): the name, the score, the
    /// highlight `new`.
    Scores,
    /// The levels' best scores (`saves::best_times::LEVEL_SCORES`): the name, the key,
    /// the highlight `mark`.
    LevelScores,
}

impl Source {
    fn need(&self, what: &'static str) -> Result<(), BadInput> {
        let there = match self {
            Source::Scores => crate::saves::best_times::SCORES.is_taken(),
            Source::LevelScores => crate::saves::best_times::LEVEL_SCORES.is_taken(),
        };
        if there { Ok(()) } else { Err(BadInput(what)) }
    }
}

/// A scores page: its table and how it is drawn.
pub struct Table {
    pub at: Source,
    /// The best scores keep the last row's y in `dword_7559B0`.
    pub keeps_y: bool,
    /// Where the group's number is: a dword, or (None) `dword_75A2D6[page]`.
    pub group_at: Option<&'static Dword>,
    /// A fixed group instead.
    pub group_fixed: Option<u32>,
    /// An entry whose highlight field is 1 is drawn in the highlight fonts (0x51E524 /
    /// 0x51E544).
    pub highlight: bool,
    /// How far below the rank's y the sprites go (and `dword_7559B0` with them).
    pub sprite_dy: i32,
}

pub const BEST_SCORES: Table = Table {
    at: Source::Scores,
    keeps_y: true,
    group_at: None,
    group_fixed: None,
    highlight: false,
    sprite_dy: 0,
};
pub const CLASS_TIMES: Table = Table {
    at: Source::LevelScores,
    keeps_y: false,
    group_at: None,
    group_fixed: None,
    highlight: false,
    sprite_dy: 0,
};
/// `screens::draw_class_times_marked`'s (0x48AB87): the class `dword_7587EC`, the player's
/// own times (`mark` 1) in the highlight fonts, the sprites 4 lower.
pub const MARKED_CLASS_TIMES: Table = Table {
    at: Source::LevelScores,
    keeps_y: true,
    group_at: Some(&crate::menu::owner::MENUS.play.arcade.level),
    group_fixed: None,
    highlight: true,
    sprite_dy: 4,
};

/// The family's screen for `rc2k_core::screens::best_scores`: the names are copied out
/// of the typed tables (`HeldText::Held`), the game's texts drawn where they are.
pub type ScoresScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::best_scores::Font,
    rc2k_core::screens::best_scores::Work,
    HeldText,
>;

/// `screens::draw_best_scores` (0x4A9300) / `screens::draw_class_times` (0x46D162): the
/// page's group, from rank 1 until ten or an entry of -1: its name (x 0xF0), its value (x
/// 0x21C), its rank (x 0xC8, the rank font), the car's sprite 0x1C + car (x 0xD2) and the
/// flag's 0x44 + flag (24 and 25 swapped, x 0x1CC) on the rank's line; each text starting at
/// the last one's y (`draw_text` leaves it), the next row 0x19 below; the sprites' palette
/// the line buffer past its NUL (ebx after the rank's `draw_text`). Every register kept.
/// Refuses (before anything) no table.
pub fn draw_scores(tables: &Tables, t: &Table, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::best_scores::scores`.
    {
        t.at.need("no best scores table")?;
        let group = match (t.group_fixed, t.group_at) {
            (Some(g), _) => g,
            (None, Some(a)) => a.get(),
            (None, None) => crate::session::read_on_in(
                &tables.lists,
                &tables.lists.page_titles_b,
                PAGE.get() as usize,
                "best scores: a game value",
            )?,
        };
        let ten = match t.at {
            Source::Scores => crate::saves::best_times::SCORES
                .with(|s| s.groups.get(group as usize).copied())?
                .ok_or(BadInput("a group past the best scores' five"))?
                .map(|e| HeldRow {
                    name: e.name_text(),
                    figure: e.score,
                    nation: e.nation,
                    car: e.car,
                    marked: e.new == 1,
                    ends: e.name_ref == u32::MAX,
                }),
            Source::LevelScores => crate::saves::best_times::LEVEL_SCORES
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
        draw_held_rows(t, &ten, canvas)
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
fn draw_held_rows(t: &Table, ten: &[HeldRow; 10], canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let fault = Fault::default();
    let row = |n: u32| held_row(&fault, &ten[n as usize], t.highlight);
    let ends = |n: u32| ten[n as usize].ends;
    let rows = Rows {
        row: &row,
        ends_at: &ends,
    };
    let mut screen = ScoresScreen::new(canvas);
    let drawn = rc2k_core::screens::best_scores::scores(&mut screen, &rows, t.keeps_y, t.sprite_dy);
    fault.check()?;
    drawn
}

/// `screens::draw_best_scores_page` (0x46D149): the best scores for pages up to 4, else
/// the class times.
pub fn draw_best_scores_page(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let tables = &menus.tables;
    if (PAGE.get() as i32) <= 4 {
        draw_scores(tables, &BEST_SCORES, canvas)
    } else {
        draw_scores(tables, &CLASS_TIMES, canvas)
    }
}

/// `screens::best_scores_caption` (0x46D128): the caption (`g_captionText`) string
/// `dword_75A2A6[page]` and its NUL. Every register kept.
pub fn best_scores_caption(menus: &Menus) -> Result<(), Refused> {
    let tables = &menus.tables;
    // Converted: `rc2k_core::screens::captions::fit` (through `car_caption::put_string`); refused
    // where the original writes past the caption's 0x100 bytes.
    let id = crate::session::read_on_in(
        &tables.lists,
        &tables.lists.page_titles_a,
        PAGE.get() as usize,
        "best scores: a game value",
    )?;
    crate::screens::car_caption::put_string(
        id,
        &crate::screens::text_buffers_store::TEXT_BUFFERS.caption,
        0x100,
    )
}

/// `screens::replay_range_caption` (0x470501): the caption "n-m", n = `dword_755248` x 20,
/// m = n + 19 (`strings::append_u32`, `append_minus`). Every register kept.
pub fn replay_range_caption(menus: &Menus) -> Result<(), BadInput> {
    // Converted: `rc2k_core::screens::best_scores::range_caption`.
    let mut bytes =
        rc2k_core::screens::best_scores::range_caption(menus.tables.pages.replay_page.get());
    bytes.push(0);
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .caption
        .write_at(0, &bytes, "the caption")
}

/// A stage times page: where its stage record is and which table holds its times.
pub struct StageTimes {
    /// The stage record's table (the records 0xA4 bytes; +0x38 the stage's name string, +0x3C
    /// its times' number) and the record's number.
    pub records: fn(&Menus) -> Result<RecordTable, BadInput>,
    pub record: &'static Dword,
    /// The times' table (ten for each stage; the record's +0x3C the stage).
    pub table: fn(&Menus) -> &'static StageTable,
    /// The rally page keeps the sprites' y (+4) in `dword_7559B0` and the flag's glyph in
    /// `dword_7559B4`.
    pub rally: bool,
}

fn champ_records(menus: &Menus) -> Result<RecordTable, BadInput> {
    let screens = &menus.screens;
    system().records.stage_tables.group_event_table(
        screens.event.stage_group.get(),
        screens.event.stage_sub.get(),
    )
}

fn champ_table(menus: &Menus) -> &'static StageTable {
    if menus.screens.event.stage_group.get() == 1 {
        &crate::saves::best_times::STAGE_TIMES_ALT
    } else {
        &crate::saves::best_times::TIME_TRIAL_TIMES
    }
}

fn rally_records(menus: &Menus) -> Result<RecordTable, BadInput> {
    system()
        .records
        .stage_tables
        .level_table(menus.play.arcade.level.get())
}

fn rally_table(_: &Menus) -> &'static StageTable {
    &crate::saves::best_times::COURSE_TIMES
}

/// `screens::draw_stage_times`' (0x48D0F2).
pub const STAGE_TIMES: StageTimes = StageTimes {
    records: champ_records,
    record: &crate::menu::owner::MENUS.screens.event.stage_number,
    table: champ_table,
    rally: false,
};
/// `screens::draw_rally_stage_times`' (0x48A948).
pub const RALLY_STAGE_TIMES: StageTimes = StageTimes {
    records: rally_records,
    record: &crate::menu::owner::MENUS.play.arcade.stage,
    table: rally_table,
    rally: true,
};

/// `screens::draw_stage_times_heading` (0x48D295) / `screens::draw_rally_times_heading`
/// (0x48AAF6): the stage's name (string +0x38 of the records) at (0x2EB, 0x77) and its stage
/// line (string 0x482 ..., built in the line buffer) at (0x2EB, 0x91), in the font at
/// 0x51E9A4. Every register kept. Refuses (before anything) no stage records.
pub fn draw_times_heading(
    menus: &Menus,
    p: &StageTimes,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::best_scores::heading`.
    {
        let base = (p.records)(menus)?;
        let id = base.title()?;
        let record = base.row(p.record.get())?;
        let line = || -> Result<Vec<u8>, Refused> { stage_line_without_nul(0x482, record) };
        rc2k_core::screens::best_scores::heading(&mut ScoresScreen::new(canvas), id, Some(&line))
    }
}

/// The stage line (`page_settings::stage_line_bytes`) without its NUL; refused past the
/// line buffer, as the originals' callers check it.
fn stage_line_without_nul(id: u32, record: RecordRef) -> Result<Vec<u8>, Refused> {
    let mut line = crate::screens::page_settings::stage_line_bytes(id, record)?;
    if line.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    line.pop();
    Ok(line)
}

/// `screens::draw_stage_times` (0x48D0F2) / `screens::draw_rally_stage_times` (0x48A948): the
/// heading, then up to ten times of the stage's group (+0x3C, its stage in the table): name
/// (x 0xF0), time (`strings::append_time`, drawn in fixed columns ending at x 0x276), rank (x
/// 0xC8), the car's and flag's sprites; the player's own in the highlight fonts; rows 0x19
/// apart from the last text's y. Every register kept. Refuses (before anything) no stage
/// records or no times table.
pub fn draw_stage_times(
    menus: &Menus,
    p: &StageTimes,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let base = (p.records)(menus)?;
    let table = (p.table)(menus);
    if !table.is_taken() {
        return Err(BadInput("no times table").into());
    }
    draw_times_heading(menus, p, canvas)?;
    let group = base.row(p.record.get())?.record()?.event_slot();
    let keep = if p.rally { Keep::Below4 } else { Keep::Nothing };
    draw_time_rows(table, group as usize, keep, canvas)?;
    Ok(())
}

pub use rc2k_core::screens::best_scores::Keep;

/// Up to ten times rows of stage `stage` of `table` (the player's own, `mark` 1,
/// highlighted). Gives whether all ten were drawn (the list did not end early: the entry
/// after the ten, the next stage's first or the block's zeros past the table, is read for
/// it). Refuses a stage past the table, and an entry after the ten past the block.
/// Converted: `rc2k_core::screens::best_scores::time_rows`.
pub fn draw_time_rows(
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
    let show_hours = crate::strings::append_time::show_hours();
    let drawn = rc2k_core::screens::best_scores::time_rows(
        &mut ScoresScreen::new(canvas),
        &rows,
        keep,
        show_hours,
    );
    fault.check()?;
    drawn
}

const SECTION: &Dword = &crate::menu::owner::MENUS.screens.event.section; // section
const INDEX: &Dword = &crate::menu::owner::MENUS.screens.event.index; // index

/// `records::find_record` of the record table: the section's record `index`.
pub fn record_at(index: u32) -> Result<RecordRef, crate::errors::BadInput> {
    crate::records::find_record::find_record_in(
        system().records.stage_tables.championship(),
        index as i32,
        SECTION.get(),
    )
}

/// Whether the record table has the section (the look-up has no end): for the refusals.
pub fn section_there() -> Result<bool, crate::errors::BadInput> {
    crate::records::find_record::has_section(
        system().records.stage_tables.championship(),
        SECTION.get(),
        0x400,
    )
}

/// `screens::draw_record_heading` (0x4A247D): the section's heading at (0x2EB, 0x77) in the
/// font at 0x51E9A4 (in event mode 0x13 `dword_75A28A[section]`, else the section's first
/// record's +0x38 string), then record `index`'s stage line (string 0x481 ...) in the line
/// buffer at (0x2EB, 0x91). Every register kept. Refuses (before anything) a section the
/// table has not.
pub fn draw_record_heading(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::best_scores::heading`.
    {
        if !section_there()? {
            return Err(BadInput("a section past the record table").into());
        }
        let first = record_at(0)?;
        let id = if menus.play.champ_view.event_mode.get() == 0x13 {
            crate::session::read_on_in(
                &menus.tables.lists,
                &menus.tables.lists.section_titles,
                SECTION.get() as usize,
                "best scores: a game value",
            )?
        } else {
            first.record()?.name()
        };
        let record = record_at(INDEX.get())?;
        let line = || -> Result<Vec<u8>, Refused> { stage_line_without_nul(0x481, record) };
        rc2k_core::screens::best_scores::heading(&mut ScoresScreen::new(canvas), id, Some(&line))
    }
}

/// `screens::draw_record_times` (0x4A41AE): the record's heading, its best times (group
/// +0x3C of `g_stageBestTimes`, the rows' y kept in `dword_7559B0`), and, when the list ran its
/// ten rows, the fastest line (string 0x1A2 at (0x142, 0x1FB), the car's sprite +0x74, the
/// name +0x54, the flag's sprite +0x78, the speed +0x30 with its unit; a list that ended early
/// leaves it out: `notes/bugs.md`). Every register kept. Refuses (before anything) a section
/// the table has not, no times table, and what `strings::append_speed` refuses.
pub fn draw_record_times(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let times = &crate::saves::best_times::STAGE_TIMES;
    let (true, true) = (section_there()?, times.is_taken()) else {
        return Err(BadInput("a section past the record table, or no times table").into());
    };
    let record = record_at(INDEX.get())?;
    // The record's fields are read at once (nothing drawn in between writes them).
    let fastest = record.record()?.holder(Holder::Fastest);
    let (top_speed, times_at, nation, car) = (
        fastest.speed,
        record.record()?.event_slot(),
        fastest.nation,
        fastest.car,
    );
    let speed = crate::strings::append_speed::append_speed(
        top_speed,
        true,
        crate::strings::units::units(),
    )?;
    draw_record_heading(menus, canvas)?;
    if !draw_time_rows(times, times_at as usize, Keep::Y, canvas)? {
        return Ok(());
    }
    rc2k_core::screens::best_scores::fastest(
        &mut ScoresScreen::new(canvas),
        nation,
        HeldText::Place(record.text(Holder::Fastest.name_at())),
        car,
        &speed,
    )
}

/// `screens::draw_event_best_scores`' (0x4A8DBF): group `dword_7558F4`, the player's own
/// (+0x14 1) in the highlight fonts.
pub const EVENT_BEST_SCORES: Table = Table {
    group_at: Some(&crate::menu::owner::MENUS.screens.event.class),
    highlight: true,
    ..BEST_SCORES
};
/// `screens::draw_overall_best_scores`' (0x4A8FA2): group 4.
pub const OVERALL_BEST_SCORES: Table = Table {
    group_fixed: Some(4),
    highlight: true,
    ..BEST_SCORES
};

/// `screens::draw_event_best_scores` (0x4A8DBF) / `screens::draw_overall_best_scores`
/// (0x4A8FA2): the heading at (0x2EB, 0x77) in the font at 0x51E9A4: in event mode 0x13
/// (`g_eventMode`) the section's name (`dword_75A28A[section]`), else string `other` (the
/// first: 0x35E + `dword_7558F4`, kept in `dword_7559EC`; the second 0x362); then the scores
/// as `draw_scores`. (Both first look the section's first record up with
/// `records::find_record` and do not use it: left out.) Every register kept. Refuses
/// (before anything) no table.
pub fn draw_scores_with_heading(
    menus: &Menus,
    t: &Table,
    event: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    t.at.need("no best scores table")?;
    // Converted: `rc2k_core::screens::best_scores::scores_heading`, `heading`.
    let section = (menus.play.champ_view.event_mode.get() == 0x13)
        .then(|| {
            crate::session::read_on_in(
                &menus.tables.lists,
                &menus.tables.lists.section_titles,
                menus.screens.event.section.get() as usize,
                "a section title",
            )
        })
        .transpose()?;
    let mut screen = ScoresScreen::new(canvas);
    let id = rc2k_core::screens::best_scores::scores_heading(
        &mut screen,
        section,
        event.then(|| menus.screens.event.class.get()),
    );
    rc2k_core::screens::best_scores::heading(&mut screen, id, None)?;
    draw_scores(&menus.tables, t, screen.canvas)
}
