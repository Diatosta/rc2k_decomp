//! The best scores pages (`menu::view_best_scores_loop`'s lists; stream A, part F1,
//! 2026-10-18): page `dword_755240` 0 .. 4 shows a group of the best scores
//! (`dword_7556D4`: groups of ten 0x3C-byte entries, 0x258 a group; +8 the nationality, +0xC
//! the car, +0x18 the name, +0x38 the score), 5 on a group of the class times (`dword_7556DC`:
//! groups of ten 0x50-byte entries, 0x320 a group; +8 the nationality, +0x10 the car, +0x14
//! the time, +0x20 the name); the group by `dword_75A2D6[page]`, the caption string by
//! `dword_75A2A6[page]`. And the replays' range caption. Converted 2026-10-20:
//! `rc2k_core::best_scores` draws through `game_screen::GameScreen`.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::screens::game_screen::GameScreen;
use crate::text::draw_text::Canvas;
use rc2k_core::best_scores::{Row, Rows};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

const PAGE: u32 = 0x755240; // dword_755240
pub const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText

/// A table's layout: where it is, its group and entry sizes, and the entry's name, value
/// and car offsets (the nationality is +8).
pub struct Table {
    pub at: u32,
    pub group: u32,
    pub entry: u32,
    pub name: u32,
    pub value: u32,
    /// The car's number (its sprite 0x44 + it, cars 0x18 / 0x19 swapped).
    pub car: u32,
    /// The best scores keep the last row's y in `dword_7559B0`.
    pub keeps_y: bool,
    /// Where the group's number is: a dword, or (None) `dword_75A2D6[page]`.
    pub group_at: Option<u32>,
    /// A fixed group instead.
    pub group_fixed: Option<u32>,
    /// An entry with this field 1 is drawn in the highlight fonts (0x51E524 / 0x51E544).
    pub highlight: Option<u32>,
    /// How far below the rank's y the sprites go (and `dword_7559B0` with them).
    pub sprite_dy: i32,
}

pub const BEST_SCORES: Table = Table {
    at: 0x7556D4,
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
    at: 0x7556DC,
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
    at: 0x7556DC,
    group: 0x320,
    entry: 0x50,
    name: 0x20,
    value: 0x14,
    car: 0x10,
    keeps_y: true,
    group_at: Some(0x7587EC),
    group_fixed: None,
    highlight: Some(0x40),
    sprite_dy: 4,
};

/// The family's screen for `rc2k_core::best_scores`.
pub type ScoresScreen<'a> =
    GameScreen<'a, rc2k_core::best_scores::Font, rc2k_core::best_scores::Work>;

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
    // Converted: `rc2k_core::best_scores::scores`.
    unsafe {
        let base = k(t.at);
        if base == 0 {
            return Err(BadInput("no best scores table").into());
        }
        let group = match (t.group_fixed, t.group_at) {
            (Some(g), _) => g,
            (None, Some(a)) => k(a),
            (None, None) => {
                ((game_addr(rva(0x75A2D6)) as u32 + 4 * k(PAGE)) as *const u32).read_unaligned()
            }
        };
        let first = base.wrapping_add(group.wrapping_mul(t.group));
        let row = rows_of(first, t.entry, (t.name, t.value, 8, t.car, t.highlight));
        let ends = ends_of(first, t.entry);
        let rows = Rows {
            row: &row,
            ends_at: &ends,
        };
        let mut screen = ScoresScreen::new(canvas);
        rc2k_core::best_scores::scores(&mut screen, &rows, t.keeps_y, t.sprite_dy)
    }
}

/// `screens::draw_best_scores_page` (0x46D149): the best scores for pages up to 4, else
/// the class times.
pub unsafe fn draw_best_scores_page(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        if (k(PAGE) as i32) <= 4 {
            draw_scores(&BEST_SCORES, canvas)
        } else {
            draw_scores(&CLASS_TIMES, canvas)
        }
    }
}

/// `screens::best_scores_caption` (0x46D128): the caption (`g_captionText`) string
/// `dword_75A2A6[page]` and its NUL. Every register kept.
pub unsafe fn best_scores_caption() {
    unsafe {
        let id = ((game_addr(rva(0x75A2A6)) as u32 + 4 * k(PAGE)) as *const u32).read_unaligned();
        let mut cursor = game_addr(CAPTION_RVA) as *mut u8;
        crate::strings::append_string_by_id::append_string_by_id(id, &mut |p, n| {
            std::ptr::copy_nonoverlapping(p, cursor, n);
            cursor = cursor.add(n);
        });
        cursor.write(0);
    }
}

/// `screens::replay_range_caption` (0x470501): the caption "n-m", n = `dword_755248` x 20,
/// m = n + 19 (`strings::append_u32`, `append_minus`). Every register kept.
pub unsafe fn replay_range_caption() {
    // Converted: `rc2k_core::best_scores::range_caption`.
    let mut bytes = rc2k_core::best_scores::range_caption(k(0x755248));
    bytes.push(0);
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            game_addr(CAPTION_RVA) as *mut u8,
            bytes.len(),
        )
    };
}

/// A stage times page: where its stage record is and which table holds its times.
pub struct StageTimes {
    /// The stage record's table (the records 0xA4 bytes; +0x38 the stage's name string, +0x3C
    /// its times' number) and the record's number.
    pub records: fn() -> u32,
    pub record: u32,
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
            ((game_addr(rva(0x735512)) as u32 + 4 * k(0x7558C8)) as *const u32).read_unaligned();
        if table == 0 {
            return 0;
        }
        ((table + 4 * k(0x7558D0)) as *const u32).read_unaligned()
    }
}

fn champ_table() -> u32 {
    if k(0x7558C8) == 1 {
        k(0x7556E4)
    } else {
        k(0x7556E8)
    }
}

fn rally_records() -> u32 {
    unsafe { ((game_addr(rva(0x73555A)) as u32 + 4 * k(0x7587EC)) as *const u32).read_unaligned() }
}

fn rally_table() -> u32 {
    k(0x7556E0)
}

/// `screens::draw_stage_times`' (0x48D0F2).
pub const STAGE_TIMES: StageTimes = StageTimes {
    records: champ_records,
    record: 0x7558CC,
    table: champ_table,
    rally: false,
};
/// `screens::draw_rally_stage_times`' (0x48A948).
pub const RALLY_STAGE_TIMES: StageTimes = StageTimes {
    records: rally_records,
    record: 0x7587F0,
    table: rally_table,
    rally: true,
};

/// `screens::draw_stage_times_heading` (0x48D295) / `screens::draw_rally_times_heading`
/// (0x48AAF6): the stage's name (string +0x38 of the records) at (0x2EB, 0x77) and its stage
/// line (string 0x482 ..., built in the line buffer) at (0x2EB, 0x91), in the font at
/// 0x51E9A4. Every register kept. Refuses (before anything) no stage records.
pub unsafe fn draw_times_heading(p: &StageTimes, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::best_scores::heading`.
    unsafe {
        let base = (p.records)();
        if base == 0 {
            return Err(BadInput("no stage records").into());
        }
        let id = ((base + 0x38) as *const u32).read_unaligned();
        let record = base.wrapping_add(k(p.record).wrapping_mul(0xA4));
        let line = || -> Result<Vec<u8>, Refused> { stage_line_without_nul(0x482, record) };
        rc2k_core::best_scores::heading(&mut ScoresScreen::new(canvas), id, Some(&line))
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
        let record = base.wrapping_add(k(p.record).wrapping_mul(0xA4));
        let group = ((record + 0x3C) as *const u32).read_unaligned();
        let entry = (p.table)().wrapping_add(group.wrapping_mul(0x618));
        let keep = if p.rally { Keep::Below4 } else { Keep::Nothing };
        draw_time_rows(entry, keep, canvas)?;
        Ok(())
    }
}

pub use rc2k_core::best_scores::Keep;

/// Up to ten times rows from `entry` (0x9C bytes each, see `StageTimes`). Gives whether all
/// ten were drawn (the list did not end early). Converted:
/// `rc2k_core::best_scores::time_rows`.
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
    rc2k_core::best_scores::time_rows(&mut ScoresScreen::new(canvas), &rows, keep, show_hours)
}

const RECORDS: u32 = 0x735576; // g_recordTable
const SECTION: u32 = 0x75595C; // section
const INDEX: u32 = 0x755958; // index

/// `records::find_record` of the record table: the section's record `index`.
pub fn record_at(index: u32) -> u32 {
    crate::records::find_record::find_record_in(
        game_addr(rva(RECORDS as usize)) as u32,
        index as i32,
        k(SECTION),
    )
}

/// Whether the record table has the section (the look-up has no end): for the refusals.
pub fn section_there() -> bool {
    crate::records::find_record::has_section(
        game_addr(rva(RECORDS as usize)) as u32,
        k(SECTION),
        0x400,
    )
}

/// `screens::draw_record_heading` (0x4A247D): the section's heading at (0x2EB, 0x77) in the
/// font at 0x51E9A4 (in event mode 0x13 `dword_75A28A[section]`, else the section's first
/// record's +0x38 string), then record `index`'s stage line (string 0x481 ...) in the line
/// buffer at (0x2EB, 0x91). Every register kept. Refuses (before anything) a section the
/// table has not.
pub unsafe fn draw_record_heading(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::best_scores::heading`.
    unsafe {
        if !section_there() {
            return Err(BadInput("a section past the record table").into());
        }
        let first = record_at(0);
        let id = if k(0x7556D0) == 0x13 {
            ((game_addr(rva(0x75A28A)) as u32 + 4 * k(SECTION)) as *const u32).read_unaligned()
        } else {
            ((first + 0x38) as *const u32).read_unaligned()
        };
        let record = record_at(k(INDEX));
        let line = || -> Result<Vec<u8>, Refused> { stage_line_without_nul(0x481, record) };
        rc2k_core::best_scores::heading(&mut ScoresScreen::new(canvas), id, Some(&line))
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
        if !section_there() || k(0x7556D8) == 0 {
            return Err(BadInput("a section past the record table, or no times table").into());
        }
        let record = record_at(k(INDEX));
        let f = |o: u32| ((record + o) as *const u32).read_unaligned();
        let speed = crate::strings::append_speed::append_speed(
            f(0x30),
            true,
            crate::strings::units::units(),
        )?;
        draw_record_heading(canvas)?;
        let entry = k(0x7556D8).wrapping_add(f(0x3C).wrapping_mul(0x618));
        if !draw_time_rows(entry, Keep::Y, canvas)? {
            return Ok(());
        }
        rc2k_core::best_scores::fastest(
            &mut ScoresScreen::new(canvas),
            f(0x74),
            record + 0x54,
            f(0x78),
            &speed,
        )
    }
}

/// Whether the record pages have what they read (for the tests).
pub unsafe fn record_page_ready() -> bool {
    unsafe {
        section_there()
            && k(0x7556D8) != 0
            && crate::records::trakpak_entry::record_entry(record_at(k(INDEX)) as *const u8).is_ok()
    }
}

/// Whether a stage times page has its records, table and track-pack entry (for the tests).
pub unsafe fn times_page_ready(p: &StageTimes) -> bool {
    unsafe {
        let base = (p.records)();
        if base == 0 || (p.table)() == 0 {
            return false;
        }
        let record = base.wrapping_add(k(p.record).wrapping_mul(0xA4));
        crate::records::trakpak_entry::record_entry(record as *const u8).is_ok()
    }
}

/// `screens::draw_event_best_scores`' (0x4A8DBF): group `dword_7558F4`, the player's own
/// (+0x14 1) in the highlight fonts.
pub const EVENT_BEST_SCORES: Table = Table {
    group_at: Some(0x7558F4),
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
        if k(t.at) == 0 {
            return Err(BadInput("no best scores table").into());
        }
        // Converted: `rc2k_core::best_scores::scores_heading`, `heading`.
        let section = (k(0x7556D0) == 0x13).then(|| {
            ((game_addr(rva(0x75A28A)) as u32 + 4 * k(0x75595C)) as *const u32).read_unaligned()
        });
        let mut screen = ScoresScreen::new(canvas);
        let id = rc2k_core::best_scores::scores_heading(
            &mut screen,
            section,
            event.then(|| k(0x7558F4)),
        );
        rc2k_core::best_scores::heading(&mut screen, id, None)?;
        draw_scores(t, screen.canvas)
    }
}
