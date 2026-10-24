//! The championship's results pages (IDA had them as code outside any function; made
//! functions 2026-10-19): ten rows of the ranked times (`g_rankedTimes`) from row
//! `g_champFirstRow`, one page each of four:
//! - `screens::draw_class_rally_results` (0x4A3376): the title (string 0x4EA, or 0x4ED with
//!   `dword_75AB20` 1, at (0x310, 0x30), font 0x51E944) and the stage's name (at (0x2EB,
//!   0x77), font 0x51E9A4: in event mode 0x13 `dword_75A28A`[section], else the section's
//!   first record's +0x38); the rally's times (+0x18); the first time the class's
//!   (`menu::find_class_time`, ebx 1); the rows of the class (`dword_7558F4`) by position
//!   (`menu::find_ranked_entry`);
//! - `screens::draw_overall_rally_results` (0x4A37E2): the same with the titles 0x4EB /
//!   0x4EE, the first time the first classed entry's (`menu::first_overall_time`, eax 2,
//!   ebx 1), the rows by position alone (`menu::find_ranked_position`); a player's row is
//!   "retired or not finished" by its +0x14 where the others' are by +0x18 (sic);
//! - `screens::draw_class_stage_results` (0x4A2111): the heading `screens::draw_record_heading`;
//!   the stage's times (+0x14); the first time the class's (`find_class_time`, ebx 0); the
//!   class's rows;
//! - `screens::draw_overall_stage_results` (0x4A2F46): as the last with the first time
//!   `first_overall_time` (eax 2, ebx 0), kept in `dword_7559D4` too (the gaps' base), the
//!   rows by position alone.
//!
//! Then each: the column headings (string 0x484 at (0xC8, 0xDC) in 0x51E5A4, 0x485 at (0xFA,
//! 0xDC) and 0x47F at (0x1F4, 0xDC) in 0x51E584), the first time to `g_firstPlayerTime`,
//! `g_champRow` from the first row to `g_champRowEnd` (first + 10), and each row from y 0xF0,
//! 0x19 apart (a player's, +4 above 0, in the highlight fonts 0x51E544 / 0x51E524 /
//! 0x51E504, else 0x51E5A4 / 0x51E584 / 0x51E564): its position (+0x44) at 0xBE, its name
//! (the text at +0x20) at 0xFA; unless it finished (+0x4C 1) string 0x4FA, or 0x4FB for a
//! time of 0x36EE8A, at 0x1EF; else its time (`text::draw_time_fixed_pitch`) at 0x258 and,
//! but for position 1, the gap (`hundredths`) after string 0x579 at 0x276 at 0x2E4; then
//! the nationality's sprite (0x1C + +8) at 0xD2 and the flag's (+0x10, 0x18 and 0x19
//! swapped, + 0x44) at 0x1AE, each with the palette the last text left (ebx). Each
//! drawer's y is edx as the text drawers leave it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In the results screens' draw lists (0x758E12 .., 0x758E2A ..). The rows are found before
//! anything is drawn: a position not there is the look-ups' fatal error, raised then. Refuses
//! (before anything) no ranked list, a section the record table has not, a name or a line past the
//! line buffer, and what the look-ups refuse.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::menu::ranked_lookups::{find_ranked_entry, find_ranked_position, first_overall_time};
use crate::screens::best_scores::{record_at, section_there};
use crate::strings::append_time::show_hours;
use crate::text::draw_text::Canvas;

const LIST_RVA: usize = rva(0x7556EC); // g_rankedTimes
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const FIRST_ROW_RVA: usize = rva(0x7556B8); // g_champFirstRow
// g_champRow
// g_champRowEnd
// g_firstPlayerTime
// hundredths
// dword_7559D4
const ALT_TITLE_RVA: usize = rva(0x75AB20); // dword_75AB20
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode
const SECTION_RVA: usize = rva(0x75595C); // section
const SECTION_NAMES_RVA: usize = rva(0x75A28A); // dword_75A28A
fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

#[derive(Clone, Copy)]
enum Heading {
    /// The title (normal, with `dword_75AB20` 1) and the stage's name.
    Titles(u32, u32),
    /// `screens::draw_record_heading`.
    Record,
}

/// What tells the four apart.
#[derive(Clone, Copy)]
pub struct Page {
    heading: Heading,
    /// The time drawn and its gap: +0x18 the rally's, +0x14 the stage's.
    time: u32,
    /// The "not timed" check's field for others' rows and for a player's.
    check: (u32, u32),
    /// The first time: true the class's (`find_class_time`), false the first classed
    /// entry's (`first_overall_time`).
    by_class: bool,
    /// The first time also in `dword_7559D4`.
    keep_overall: bool,
}

pub const CLASS_RALLY: Page = Page {
    heading: Heading::Titles(0x4EA, 0x4ED),
    time: 0x18,
    check: (0x18, 0x18),
    by_class: true,
    keep_overall: false,
};
pub const OVERALL_RALLY: Page = Page {
    heading: Heading::Titles(0x4EB, 0x4EE),
    time: 0x18,
    check: (0x18, 0x14),
    by_class: false,
    keep_overall: false,
};
pub const CLASS_STAGE: Page = Page {
    heading: Heading::Record,
    time: 0x14,
    check: (0x14, 0x14),
    by_class: true,
    keep_overall: false,
};
pub const OVERALL_STAGE: Page = Page {
    heading: Heading::Record,
    time: 0x14,
    check: (0x14, 0x14),
    by_class: false,
    keep_overall: true,
};

/// The Rust version of one page.
pub unsafe fn draw_results_page(page: &Page, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let (list, class) = (g(LIST_RVA), g(CLASS_RVA));
    if list == 0 {
        return Err(BadInput("no ranked list").into());
    }
    if !section_there() {
        return Err(BadInput("a section past the record table").into());
    }
    let rally = page.time == 0x18;
    let first = if page.by_class {
        crate::menu::find_class_time::find_class_time(list, class, rally)?
    } else {
        first_overall_time(list, 2, rally)?
    };
    let first_row = g(FIRST_ROW_RVA);
    let end = first_row.wrapping_add(10);
    // every row found before anything is drawn
    let mut rows = Vec::new();
    let mut row = first_row;
    loop {
        rows.push(if page.by_class {
            find_ranked_entry(list, row, class)?
        } else {
            find_ranked_position(list, row)?
        });
        row = row.wrapping_add(1);
        if row == end || rows.len() > 0x400 {
            break;
        }
    }
    // Converted: `rc2k_core::screens::results_pages` (the rows' entries read here, the record
    // heading drawn here first).
    use rc2k_core::screens::results_pages::{
        Heading as Drawn, NOT_TIMED as NONE, Row, results_page,
    };
    unsafe {
        let heading = match page.heading {
            Heading::Titles(normal, alternative) => Drawn::Titles {
                title: if g(ALT_TITLE_RVA) == 1 {
                    alternative
                } else {
                    normal
                },
                name: if g(EVENT_MODE_RVA) == 0x13 {
                    g(SECTION_NAMES_RVA + 4 * g(SECTION_RVA) as usize)
                } else {
                    get(record_at(0) + 0x38)
                },
            },
            Heading::Record => {
                crate::screens::best_scores::draw_record_heading(canvas)?;
                Drawn::Drawn
            }
        };
        let rows: Vec<Row<u32>> = rows
            .into_iter()
            .map(|e| {
                let player = (get(e + 4) as i32) > 0;
                let check = if player { page.check.1 } else { page.check.0 };
                Row {
                    player,
                    position: get(e + 0x44),
                    name: e + 0x20,
                    finished: get(e + 0x4C) == 1,
                    not_timed: get(e + check) == NONE,
                    time: get(e + page.time),
                    nation: get(e + 8),
                    car: get(e + 0x10),
                }
            })
            .collect();
        results_page(
            &mut crate::screens::game_screen::GameScreen::new(canvas),
            heading,
            first,
            first_row,
            page.keep_overall,
            show_hours(),
            &rows,
        )?;
    }
    Ok(())
}

/// `results_page`: the pages the typed tables run.
macro_rules! results_page_entry {
    ($module:ident, $name:literal, $address:expr, $page:expr) => {
        pub mod $module {
            use super::*;
            /// The Rust version the typed tables run (`menu::table_calls`).
            pub fn draw() -> Result<(), Refused> {
                unsafe { draw_results_page(&$page, &mut crate::text::draw_text::GameCanvas) }
            }
        }
    };
}

results_page_entry!(
    class_rally,
    "screens::draw_class_rally_results",
    0x4A3376,
    CLASS_RALLY
);
results_page_entry!(
    overall_rally,
    "screens::draw_overall_rally_results",
    0x4A37E2,
    OVERALL_RALLY
);
results_page_entry!(
    class_stage,
    "screens::draw_class_stage_results",
    0x4A2111,
    CLASS_STAGE
);
results_page_entry!(
    overall_stage,
    "screens::draw_overall_stage_results",
    0x4A2F46,
    OVERALL_STAGE
);
