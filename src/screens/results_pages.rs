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
//! anything is drawn: a position not there is the look-ups' fatal error, raised then.
//! Refuses (before anything) no ranked list, a section the record table has not, a name or a
//! line past the line buffer, and what the look-ups refuse. 0x4A2111 and 0x4A2F46 call in
//! their first bytes: manual patches with private copies (every call relocated). Compared
//! in shadow mode (`dispatch_draw`). Self-test: a list of the test's (two classes, players,
//! a retired and an unfinished row), from rows 1 and 2, `dword_75AB20` 0 and 1, nothing
//! drawn (skipped without the fonts or the record table's section).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::ranked_lookups::{find_ranked_entry, find_ranked_position, first_overall_time};
use crate::screens::LINE_BUFFER_RVA;
use crate::screens::best_scores::{record_at, section_there};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::append_time::show_hours;
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

const LIST_RVA: usize = rva(0x7556EC); // g_rankedTimes
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const FIRST_ROW_RVA: usize = rva(0x7556B8); // g_champFirstRow
const ROW_RVA: usize = rva(0x755579); // g_champRow
const ROW_END_RVA: usize = rva(0x7556BC); // g_champRowEnd
const FIRST_TIME_RVA: usize = rva(0x7559BC); // g_firstPlayerTime
const GAP_RVA: usize = rva(0x7559C0); // hundredths
const OVERALL_FIRST_RVA: usize = rva(0x7559D4); // dword_7559D4
const ALT_TITLE_RVA: usize = rva(0x75AB20); // dword_75AB20
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode
const SECTION_RVA: usize = rva(0x75595C); // section
const SECTION_NAMES_RVA: usize = rva(0x75A28A); // dword_75A28A
const LINE_LEN: usize = 0x100;
const NOT_TIMED: u32 = 0x36EE8A;

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

fn written() -> Vec<Region> {
    vec![
        (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
        at(FIRST_TIME_RVA),
        at(GAP_RVA),
        at(OVERALL_FIRST_RVA),
        at(ROW_RVA),
        at(ROW_END_RVA),
    ]
}

fn state() -> Vec<u8> {
    capture(&written())
}

/// A list of the test's: 24 entries, classes 2 and 3 in turn, positions 1 .. 12 in each,
/// some players, a retired row and one not timed, the names as text; its end mark.
fn results_list() -> Vec<u32> {
    let words = crate::menu::ranked_lookups::ENTRY_LEN as usize / 4;
    let mut list = vec![0u32; 25 * words];
    for n in 0..24usize {
        let w = &mut list[n * words..(n + 1) * words];
        w[0] = n as u32;
        w[1] = if n % 7 == 3 { 1 } else { 0 };
        w[2] = n as u32 % 9;
        w[4] = [0x18u32, 0x19, 3, 0x20][n % 4];
        w[0x14 / 4] = 5000 + 311 * n as u32;
        w[0x18 / 4] = 9000 + 577 * n as u32;
        w[0x1C / 4] = [2, 3][n % 2];
        let name = format!("Driver {n}");
        for (k, b) in name.bytes().enumerate() {
            let at = 0x20 + k;
            w[at / 4] |= (b as u32) << (8 * (at % 4));
        }
        w[0x44 / 4] = n as u32 / 2 + 1;
        w[0x4C / 4] = if n == 9 || n == 12 { 0 } else { 1 };
        if n == 12 {
            w[0x14 / 4] = NOT_TIMED;
            w[0x18 / 4] = NOT_TIMED;
        }
    }
    list[24 * words] = u32::MAX;
    list
}

fn test(
    hook: &RawHook,
    call_original: crate::screens::harness::CallOriginal,
    rust: &dyn Fn(&Regs, &mut dyn Canvas) -> Result<Left, Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    if !section_there() {
        return t.skip("the record table has not the section (no stage chosen yet)");
    }
    let inputs: Vec<Region> = vec![
        at(LIST_RVA),
        at(CLASS_RVA),
        at(FIRST_ROW_RVA),
        at(ALT_TITLE_RVA),
    ];
    let saved = (capture(&inputs), state());
    let list = results_list();
    let base = list.as_ptr() as u32;
    for (class, first, alt) in [(2u32, 1u32, 0u32), (3, 2, 1), (2, 2, 0)] {
        check_draw(
            &mut t,
            &format!("class {class}, from row {first}, the other title {alt}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(LIST_RVA, base);
                game_write(CLASS_RVA, class);
                game_write(FIRST_ROW_RVA, first);
                game_write(ALT_TITLE_RVA, alt);
            },
            rust,
            &state,
        );
    }
    restore(&inputs, &saved.0);
    restore(&written(), &saved.1);
    drop(list);
    t.finish()
}

/// A hook for one page.
macro_rules! results_page {
    ($module:ident, $name:literal, $address:expr, $page:expr $(, copy = $len:expr, $relocate:expr)?) => {
        pub mod $module {
            use super::*;
            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
            pub static HOOK: RawHook = RawHook::new($name, rva($address), thunk, &ORIGINAL)
                $(.private_copy($len).relocate_calls(&$relocate).manual_patch())?
                .self_test(self_test);
            hook_asm!(thunk, call_original, dispatch = dispatch, original = ORIGINAL);
            fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
                unsafe { draw_results_page(&$page, canvas) }?;
                Ok(Left::nothing())
            }
            extern "C" fn dispatch(r: &mut Regs) {
                dispatch_draw(&HOOK, r, call_original, &rust, &state);
            }
            fn self_test() -> (usize, usize) {
                test(&HOOK, call_original, &rust)
            }
        }
    };
}

results_page!(
    class_rally,
    "screens::draw_class_rally_results",
    0x4A3376,
    CLASS_RALLY
);
results_page!(
    overall_rally,
    "screens::draw_overall_rally_results",
    0x4A37E2,
    OVERALL_RALLY
);
results_page!(
    class_stage,
    "screens::draw_class_stage_results",
    0x4A2111,
    CLASS_STAGE,
    copy = 0x32F,
    [
        0x1, 0x1B, 0x35, 0x4F, 0x64, 0x9E, 0xB6, 0xBB, 0xD3, 0xE8, 0x10E, 0x12A, 0x13D, 0x142,
        0x15A, 0x189, 0x199, 0x19E, 0x1B6, 0x1C9, 0x1CE, 0x1E6, 0x1FB, 0x221, 0x23D, 0x250, 0x255,
        0x26D, 0x298, 0x2A8, 0x2AD, 0x2C5, 0x2E2, 0x30B
    ]
);
results_page!(
    overall_stage,
    "screens::draw_overall_stage_results",
    0x4A2F46,
    OVERALL_STAGE,
    copy = 0x334,
    [
        0x1, 0x1B, 0x35, 0x4F, 0x64, 0xA3, 0xBB, 0xC0, 0xD8, 0xED, 0x113, 0x12F, 0x142, 0x147,
        0x15F, 0x18E, 0x19E, 0x1A3, 0x1BB, 0x1CE, 0x1D3, 0x1EB, 0x200, 0x226, 0x242, 0x255, 0x25A,
        0x272, 0x29D, 0x2AD, 0x2B2, 0x2CA, 0x2E7, 0x310
    ]
);

/// The four, for the registry.
pub static HOOKS: [&dyn crate::framework::GameHook; 4] = [
    &class_rally::HOOK,
    &overall_rally::HOOK,
    &class_stage::HOOK,
    &overall_stage::HOOK,
];
