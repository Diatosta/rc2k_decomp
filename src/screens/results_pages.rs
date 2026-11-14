//! The championship's results pages: ten rows of the ranked times from the first row,
//! one page each of four:
//! - `draw_class_rally_results`: the title (string 0x4EA, or 0x4ED with `state.menus.rally_done`
//!   set), the stage's name (in event mode 0x13 the section's entry of the heading list, else the
//!   section's first record), the rally's times, the first time the class's, and the rows of the
//!   class by position.
//! - `draw_overall_rally_results`: the same with titles 0x4EB / 0x4EE, the first time the first
//!   classed entry's and the rows by position alone. A player's row is "retired or not finished"
//!   by its +0x14 where the others' are by +0x18 (kept as is).
//! - `draw_class_stage_results`: the heading `screens::draw_record_heading`, the stage's times,
//!   the class's first time and rows.
//! - `draw_overall_stage_results`: as the last with the overall first time (the gaps' base) and
//!   the rows by position alone.
//!
//! Each then draws the column headings and per row the position, name, time (string 0x4FA, or
//! 0x4FB for a time of 0x36EE8A, when the car did not finish) and, but for position 1, the gap
//! after string 0x579, then the nationality's and the flag's sprites; a player's own row is in the
//! highlight fonts. The rows are found before anything is drawn: a position not there is the
//! look-ups' fatal error. Refuses no ranked list, a section the record table lacks, a name or a
//! line past the line buffer and what the look-ups refuse.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::lists;
use crate::menu::ranked_lookups::{find_ranked_entry, find_ranked_position, first_overall_time};
use crate::screens::best_scores::{record_at, section_there};
use crate::screens::game_screen::{HeldText, held_name};
use crate::strings::append_time::show_hours;
use crate::text::draw_text::Canvas;

#[derive(Clone, Copy)]
enum Heading {
    /// The title (normal, with `rally_done` set) and the stage's name.
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
    /// The first time also kept as the overall first time.
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

/// Draws one page.
pub fn draw_results_page(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    page: &Page,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let class = menus.screens.event.class.get();
    if !section_there(menus, system)? {
        return Err(BadInput("a section past the record table").into());
    }
    let rally = page.time == 0x18;
    let first = if page.by_class {
        crate::menu::find_class_time::find_class_time(class, rally)?
    } else {
        first_overall_time(2, rally)?
    };
    let first_row = menus.play.champ_view.first_row.get();
    let end = first_row.wrapping_add(10);
    // every row found before anything is drawn
    let mut rows = Vec::new();
    let mut row = first_row;
    loop {
        rows.push(if page.by_class {
            find_ranked_entry(row, class)?
        } else {
            find_ranked_position(row)?
        });
        row = row.wrapping_add(1);
        if row == end || rows.len() > 0x400 {
            break;
        }
    }
    // The logic is in `rc2k_core::screens::results_pages` (the rows' entries read here, the record
    // heading drawn here first).
    use rc2k_core::screens::results_pages::{
        Heading as Drawn, NOT_TIMED as NONE, Row, results_page,
    };
    {
        let heading = match page.heading {
            Heading::Titles(normal, alternative) => Drawn::Titles {
                title: if menus.screens.state.menus.rally_done.is_set() {
                    alternative
                } else {
                    normal
                },
                name: if menus.play.champ_view.event_mode.get() == 0x13 {
                    crate::screens::stage_panel::section_title(
                        lists(),
                        menus.screens.event.section.get(),
                    )?
                } else {
                    record_at(menus, system, 0)?
                        .record(&system.records().stage_tables)?
                        .name()
                },
            },
            Heading::Record => {
                crate::screens::best_scores::draw_record_heading(o, system, canvas)?;
                Drawn::Drawn
            }
        };
        // A time field of the page's (+0x14 the time, +0x18 the total).
        let time_at = |e: &rc2k_core::saves::entries::BestTime, at: u32| {
            if at == 0x18 {
                e.head.rally_time
            } else {
                e.head.key
            }
        };
        let rows: Vec<Row<HeldText>> = rows
            .into_iter()
            .map(|e| {
                let player = (e.head.player as i32) > 0;
                let check = if player { page.check.1 } else { page.check.0 };
                Ok(Row {
                    player,
                    position: e.head.figure,
                    name: held_name(e.name_text())?,
                    finished: e.head.entered == 1,
                    not_timed: time_at(&e, check) == NONE,
                    time: time_at(&e, page.time),
                    nation: e.head.nation,
                    car: e.head.car,
                })
            })
            .collect::<Result<_, BadInput>>()?;
        results_page(
            &mut crate::screens::game_screen::GameScreen::<_, _, HeldText>::new(canvas, o),
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
            /// Run by the typed tables (`menu::table_calls`).
            pub fn draw(
                o: crate::owners::Owners<'static>,
                system: &crate::system::owner::System,
                gfx: &'static crate::gfx::owner::Gfx,
            ) -> Result<(), Refused> {
                draw_results_page(
                    o,
                    system,
                    &$page,
                    &mut crate::text::draw_text::GameCanvas::new(gfx),
                )
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
