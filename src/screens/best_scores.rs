//! The best scores pages (`menu::view_best_scores_loop`'s lists; stream A, part F1,
//! 2026-10-18): page `dword_755240` 0 .. 4 shows a group of the best scores
//! (`dword_7556D4`: groups of ten 0x3C-byte entries, 0x258 a group; +8 the nationality, +0xC
//! the car, +0x18 the name, +0x38 the score), 5 on a group of the class times (`dword_7556DC`:
//! groups of ten 0x50-byte entries, 0x320 a group; +8 the nationality, +0x10 the car, +0x14
//! the time, +0x20 the name); the group by `dword_75A2D6[page]`, the caption string by
//! `dword_75A2A6[page]`. And the replays' range caption. Converted 2026-10-20:
//! `rc2k_core::screens::best_scores` draws through `game_screen::GameScreen`.

use crate::errors::{BadInput, Refused};
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::pages_store::MENU_PAGES;
use crate::screens::game_screen::GameScreen;
use crate::session::Dword;
use crate::text::draw_text::Canvas;
use rc2k_core::screens::best_scores::{Row, Rows};

const PAGE: &Dword = &MENU_PAGES.scores_page; // dword_755240
/// A table's layout: where it is, its group and entry sizes, and the entry's name, value
/// and car offsets (the nationality is +8).
pub struct Table {
    pub at: &'static Dword,
    pub group: u32,
    pub entry: u32,
    pub name: u32,
    pub value: u32,
    /// The car's number (its sprite 0x44 + it, cars 0x18 / 0x19 swapped).
    pub car: u32,
    /// The best scores keep the last row's y in `dword_7559B0`.
    pub keeps_y: bool,
    /// Where the group's number is: a dword, or (None) `dword_75A2D6[page]`.
    pub group_at: Option<&'static Dword>,
    /// A fixed group instead.
    pub group_fixed: Option<u32>,
    /// An entry with this field 1 is drawn in the highlight fonts (0x51E524 / 0x51E544).
    pub highlight: Option<u32>,
    /// How far below the rank's y the sprites go (and `dword_7559B0` with them).
    pub sprite_dy: i32,
}

pub const BEST_SCORES: Table = Table {
    at: &CHAMP_VIEW.best_scores,
    group: 0x258,
    entry: 0x3C,
    name: 0x18,
    value: 0x38,
    car: 0xC,
    keeps_y: true,
    group_at: None,
    group_fixed: None,
    highlight: None,
    sprite_dy: 0,
};
pub const CLASS_TIMES: Table = Table {
    at: &CHAMP_VIEW.level_best_scores,
    group: 0x320,
    entry: 0x50,
    name: 0x20,
    value: 0x14,
    car: 0x10,
    keeps_y: false,
    group_at: None,
    group_fixed: None,
    highlight: None,
    sprite_dy: 0,
};
/// `screens::draw_class_times_marked`'s (0x48AB87): the class `dword_7587EC`, the player's
/// own times (+0x40 1) in the highlight fonts, the sprites 4 lower.
pub const MARKED_CLASS_TIMES: Table = Table {
    at: &CHAMP_VIEW.level_best_scores,
    group: 0x320,
    entry: 0x50,
    name: 0x20,
    value: 0x14,
    car: 0x10,
    keeps_y: true,
    group_at: Some(&ARCADE_GAME.level),
    group_fixed: None,
    highlight: Some(0x40),
    sprite_dy: 4,
};

/// The family's screen for `rc2k_core::screens::best_scores`.
pub type ScoresScreen<'a> =
    GameScreen<'a, rc2k_core::screens::best_scores::Font, rc2k_core::screens::best_scores::Work>;

/// The rows of a list of `size`-byte entries from `first`, as the core reads them.
fn rows_of(
    first: u32,
    size: u32,
    t: (u32, u32, u32, u32, Option<u32>),
) -> impl Fn(u32) -> Row<u32> {
    let (name, figure, nation, car, highlight) = t;
    move |n| {
        let entry = first.wrapping_add(n.wrapping_mul(size));
        let f = |o: u32| unsafe { ((entry + o) as *const u32).read_unaligned() };
        Row {
            name: entry + name,
            figure: f(figure),
            nation: f(nation),
            car: f(car),
            marked: highlight.is_some_and(|o| f(o) == 1),
        }
    }
}

/// Whether the list from `first` ends at entry `n` (its first dword -1).
fn ends_of(first: u32, size: u32) -> impl Fn(u32) -> bool {
    move |n| unsafe {
        (first.wrapping_add(n.wrapping_mul(size)) as *const u32).read_unaligned() == u32::MAX
    }
}

/// `screens::draw_best_scores` (0x4A9300) / `screens::draw_class_times` (0x46D162): the
/// page's group, from rank 1 until ten or an entry of -1: its name (x 0xF0), its value (x
/// 0x21C), its rank (x 0xC8, the rank font), the car's sprite 0x1C + car (x 0xD2) and the
/// flag's 0x44 + flag (24 and 25 swapped, x 0x1CC) on the rank's line; each text starting at
/// the last one's y (`draw_text` leaves it), the next row 0x19 below; the sprites' palette
/// the line buffer past its NUL (ebx after the rank's `draw_text`). Every register kept.
/// Refuses (before anything) no table.
pub unsafe fn draw_scores(t: &Table, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::best_scores::scores`.
    unsafe {
        let base = t.at.get();
        if base == 0 {
            return Err(BadInput("no best scores table").into());
        }
        let group = match (t.group_fixed, t.group_at) {
            (Some(g), _) => g,
            (None, Some(a)) => a.get(),
            (None, None) => ((MENU_LISTS.page_titles_b[0].addr() as u32 + 4 * PAGE.get())
                as *const u32)
                .read_unaligned(),
        };
        let first = base.wrapping_add(group.wrapping_mul(t.group));
        let row = rows_of(first, t.entry, (t.name, t.value, 8, t.car, t.highlight));
        let ends = ends_of(first, t.entry);
        let rows = Rows {
            row: &row,
            ends_at: &ends,
        };
        let mut screen = ScoresScreen::new(canvas);
        rc2k_core::screens::best_scores::scores(&mut screen, &rows, t.keeps_y, t.sprite_dy)
    }
}

/// `screens::draw_best_scores_page` (0x46D149): the best scores for pages up to 4, else
/// the class times.
pub unsafe fn draw_best_scores_page(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        if (PAGE.get() as i32) <= 4 {
            draw_scores(&BEST_SCORES, canvas)
        } else {
            draw_scores(&CLASS_TIMES, canvas)
        }
    }
}

/// `screens::best_scores_caption` (0x46D128): the caption (`g_captionText`) string
/// `dword_75A2A6[page]` and its NUL. Every register kept.
pub unsafe fn best_scores_caption() -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::captions::fit` (through `car_caption::put_string`); refused
    // where the original writes past the caption's 0x100 bytes.
    unsafe {
        let id = ((MENU_LISTS.page_titles_a[0].addr() as u32 + 4 * PAGE.get()) as *const u32)
            .read_unaligned();
        crate::screens::car_caption::put_string(
            id,
            crate::screens::text_buffers_store::TEXT_BUFFERS
                .caption
                .addr(),
            0x100,
        )
    }
}

/// `screens::replay_range_caption` (0x470501): the caption "n-m", n = `dword_755248` x 20,
/// m = n + 19 (`strings::append_u32`, `append_minus`). Every register kept.
pub unsafe fn replay_range_caption() {
    // Converted: `rc2k_core::screens::best_scores::range_caption`.
    let mut bytes = rc2k_core::screens::best_scores::range_caption(MENU_PAGES.replay_page.get());
    bytes.push(0);
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            crate::screens::text_buffers_store::TEXT_BUFFERS
                .caption
                .addr() as *mut u8,
            bytes.len(),
        )
    };
}

/// A stage times page: where its stage record is and which table holds its times.
pub struct StageTimes {
    /// The stage record's table (the records 0xA4 bytes; +0x38 the stage's name string, +0x3C
    /// its times' number) and the record's number.
    pub records: fn() -> u32,
    pub record: &'static Dword,
    /// The times' table (groups of 0x618 bytes; entries 0x9C bytes: +8 the nationality, +0x10
    /// the car, +0x14 the time, +0x20 the name, +0x40 the player's own).
    pub table: fn() -> u32,
    /// The rally page keeps the sprites' y (+4) in `dword_7559B0` and the flag's glyph in
    /// `dword_7559B4`.
    pub rally: bool,
}

fn champ_records() -> u32 {
    unsafe {
        let table =
            crate::records::stages_store::STAGE_TABLES.group_list(EVENT_STATE.stage_group.get());
        if table == 0 {
            return 0;
        }
        ((table + 4 * EVENT_STATE.stage_sub.get()) as *const u32).read_unaligned()
    }
}

fn champ_table() -> u32 {
    if EVENT_STATE.stage_group.get() == 1 {
        CHAMP_VIEW.stage_best_times_alt.get()
    } else {
        CHAMP_VIEW.time_trial_best_times.get()
    }
}

fn rally_records() -> u32 {
    crate::records::stages_store::STAGE_TABLES.level_table(ARCADE_GAME.level.get())
}

fn rally_table() -> u32 {
    CHAMP_VIEW.course_best_times.get()
}

/// `screens::draw_stage_times`' (0x48D0F2).
pub const STAGE_TIMES: StageTimes = StageTimes {
    records: champ_records,
    record: &EVENT_STATE.stage_number,
    table: champ_table,
    rally: false,
};
/// `screens::draw_rally_stage_times`' (0x48A948).
pub const RALLY_STAGE_TIMES: StageTimes = StageTimes {
    records: rally_records,
    record: &ARCADE_GAME.stage,
    table: rally_table,
    rally: true,
};

/// `screens::draw_stage_times_heading` (0x48D295) / `screens::draw_rally_times_heading`
/// (0x48AAF6): the stage's name (string +0x38 of the records) at (0x2EB, 0x77) and its stage
/// line (string 0x482 ..., built in the line buffer) at (0x2EB, 0x91), in the font at
/// 0x51E9A4. Every register kept. Refuses (before anything) no stage records.
pub unsafe fn draw_times_heading(p: &StageTimes, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::best_scores::heading`.
    unsafe {
        let base = (p.records)();
        if base == 0 {
            return Err(BadInput("no stage records").into());
        }
        let id = ((base + 0x38) as *const u32).read_unaligned();
        let record = base.wrapping_add(p.record.get().wrapping_mul(0xA4));
        let line = || -> Result<Vec<u8>, Refused> { stage_line_without_nul(0x482, record) };
        rc2k_core::screens::best_scores::heading(&mut ScoresScreen::new(canvas), id, Some(&line))
    }
}

/// The stage line (`page_settings::stage_line_bytes`) without its NUL; refused past the
/// line buffer, as the originals' callers check it.
fn stage_line_without_nul(id: u32, record: u32) -> Result<Vec<u8>, Refused> {
    let mut line = unsafe { crate::screens::page_settings::stage_line_bytes(id, record) }?;
    if line.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    line.pop();
    Ok(line)
}

/// `screens::draw_stage_times` (0x48D0F2) / `screens::draw_rally_stage_times` (0x48A948): the
/// heading, then up to ten times of the stage's group (+0x3C x 0x618 into the table): name
/// (x 0xF0), time (`strings::append_time`, drawn in fixed columns ending at x 0x276), rank (x
/// 0xC8), the car's and flag's sprites; the player's own in the highlight fonts; rows 0x19
/// apart from the last text's y. Every register kept. Refuses (before anything) no stage
/// records or no times table.
pub unsafe fn draw_stage_times(p: &StageTimes, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let base = (p.records)();
        if base == 0 || (p.table)() == 0 {
            return Err(BadInput("no stage records or times table").into());
        }
        draw_times_heading(p, canvas)?;
        let record = base.wrapping_add(p.record.get().wrapping_mul(0xA4));
        let group = ((record + 0x3C) as *const u32).read_unaligned();
        let entry = (p.table)().wrapping_add(group.wrapping_mul(0x618));
        let keep = if p.rally { Keep::Below4 } else { Keep::Nothing };
        draw_time_rows(entry, keep, canvas)?;
        Ok(())
    }
}

pub use rc2k_core::screens::best_scores::Keep;

/// Up to ten times rows from `entry` (0x9C bytes each, see `StageTimes`). Gives whether all
/// ten were drawn (the list did not end early). Converted:
/// `rc2k_core::screens::best_scores::time_rows`.
pub unsafe fn draw_time_rows(
    entry: u32,
    keep: Keep,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let row = rows_of(entry, 0x9C, (0x20, 0x14, 8, 0x10, Some(0x40)));
    let ends = ends_of(entry, 0x9C);
    let rows = Rows {
        row: &row,
        ends_at: &ends,
    };
    let show_hours = crate::strings::append_time::show_hours();
    rc2k_core::screens::best_scores::time_rows(
        &mut ScoresScreen::new(canvas),
        &rows,
        keep,
        show_hours,
    )
}

const SECTION: &Dword = &EVENT_STATE.section; // section
const INDEX: &Dword = &EVENT_STATE.index; // index

/// `records::find_record` of the record table: the section's record `index`.
pub fn record_at(index: u32) -> u32 {
    crate::records::find_record::find_record_in(
        crate::records::stages_store::STAGE_TABLES.championship(),
        index as i32,
        SECTION.get(),
    )
}

/// Whether the record table has the section (the look-up has no end): for the refusals.
pub fn section_there() -> bool {
    crate::records::find_record::has_section(
        crate::records::stages_store::STAGE_TABLES.championship(),
        SECTION.get(),
        0x400,
    )
}

/// `screens::draw_record_heading` (0x4A247D): the section's heading at (0x2EB, 0x77) in the
/// font at 0x51E9A4 (in event mode 0x13 `dword_75A28A[section]`, else the section's first
/// record's +0x38 string), then record `index`'s stage line (string 0x481 ...) in the line
/// buffer at (0x2EB, 0x91). Every register kept. Refuses (before anything) a section the
/// table has not.
pub unsafe fn draw_record_heading(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::best_scores::heading`.
    unsafe {
        if !section_there() {
            return Err(BadInput("a section past the record table").into());
        }
        let first = record_at(0);
        let id = if CHAMP_VIEW.event_mode.get() == 0x13 {
            ((MENU_LISTS.section_titles[0].addr() as u32 + 4 * SECTION.get()) as *const u32)
                .read_unaligned()
        } else {
            ((first + 0x38) as *const u32).read_unaligned()
        };
        let record = record_at(INDEX.get());
        let line = || -> Result<Vec<u8>, Refused> { stage_line_without_nul(0x481, record) };
        rc2k_core::screens::best_scores::heading(&mut ScoresScreen::new(canvas), id, Some(&line))
    }
}

/// `screens::draw_record_times` (0x4A41AE): the record's heading, its best times (group
/// +0x3C of `dword_7556D8`, the rows' y kept in `dword_7559B0`), and, when the list ran its
/// ten rows, the fastest line (string 0x1A2 at (0x142, 0x1FB), the car's sprite +0x74, the
/// name +0x54, the flag's sprite +0x78, the speed +0x30 with its unit; a list that ended early
/// leaves it out: `notes/bugs.md`). Every register kept. Refuses (before anything) a section
/// the table has not, no times table, and what `strings::append_speed` refuses.
pub unsafe fn draw_record_times(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        if !section_there() || CHAMP_VIEW.stage_best_times.get() == 0 {
            return Err(BadInput("a section past the record table, or no times table").into());
        }
        let record = record_at(INDEX.get());
        let f = |o: u32| ((record + o) as *const u32).read_unaligned();
        let speed = crate::strings::append_speed::append_speed(
            f(0x30),
            true,
            crate::strings::units::units(),
        )?;
        draw_record_heading(canvas)?;
        let entry = CHAMP_VIEW
            .stage_best_times
            .get()
            .wrapping_add(f(0x3C).wrapping_mul(0x618));
        if !draw_time_rows(entry, Keep::Y, canvas)? {
            return Ok(());
        }
        rc2k_core::screens::best_scores::fastest(
            &mut ScoresScreen::new(canvas),
            f(0x74),
            record + 0x54,
            f(0x78),
            &speed,
        )
    }
}

/// `screens::draw_event_best_scores`' (0x4A8DBF): group `dword_7558F4`, the player's own
/// (+0x14 1) in the highlight fonts.
pub const EVENT_BEST_SCORES: Table = Table {
    group_at: Some(&EVENT_STATE.class),
    highlight: Some(0x14),
    ..BEST_SCORES
};
/// `screens::draw_overall_best_scores`' (0x4A8FA2): group 4.
pub const OVERALL_BEST_SCORES: Table = Table {
    group_fixed: Some(4),
    highlight: Some(0x14),
    ..BEST_SCORES
};

/// `screens::draw_event_best_scores` (0x4A8DBF) / `screens::draw_overall_best_scores`
/// (0x4A8FA2): the heading at (0x2EB, 0x77) in the font at 0x51E9A4: in event mode 0x13
/// (`g_eventMode`) the section's name (`dword_75A28A[section]`), else string `other` (the
/// first: 0x35E + `dword_7558F4`, kept in `dword_7559EC`; the second 0x362); then the scores
/// as `draw_scores`. (Both first look the section's first record up with
/// `records::find_record` and do not use it: left out.) Every register kept. Refuses
/// (before anything) no table.
pub unsafe fn draw_scores_with_heading(
    t: &Table,
    event: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        if t.at.get() == 0 {
            return Err(BadInput("no best scores table").into());
        }
        // Converted: `rc2k_core::screens::best_scores::scores_heading`, `heading`.
        let section = (CHAMP_VIEW.event_mode.get() == 0x13).then(|| {
            ((MENU_LISTS.section_titles[0].addr() as u32 + 4 * EVENT_STATE.section.get())
                as *const u32)
                .read_unaligned()
        });
        let mut screen = ScoresScreen::new(canvas);
        let id = rc2k_core::screens::best_scores::scores_heading(
            &mut screen,
            section,
            event.then(|| EVENT_STATE.class.get()),
        );
        rc2k_core::screens::best_scores::heading(&mut screen, id, None)?;
        draw_scores(t, screen.canvas)
    }
}
