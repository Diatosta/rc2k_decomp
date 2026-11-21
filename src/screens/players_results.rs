//! The players' stage results pages, with the points column (`draw_players_results_points`) and
//! without (`draw_players_results`). Both draw the same columns, one row per player in the order
//! of `results.order`, one per player; the differences are the `Layout`.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::results_store::Results;
use crate::records::stages_store::{RecordRef, RecordTable};
use crate::text::draw_text::Canvas;

pub use rc2k_core::screens::players_results::{Layout, WITH_POINTS, WITHOUT_POINTS};

fn rows(menus: &Menus) -> usize {
    rc2k_core::screens::players_results::rows(menus.screens.event.read().player_count)
}

fn offset(results: &Results, row: usize) -> u32 {
    results.read().order[row]
}

/// The row's player record, refused for an offset past the four or a missing record or name.
fn player<'a>(
    menus: &'a Menus,
    results: &Results,
    row: usize,
) -> Result<crate::menu::standings::PlayerCell<'a>, Refused> {
    let o = offset(results, row);
    if o >= 16 || !o.is_multiple_of(4) {
        return Err(BadInput("a row's offset past the four players").into());
    }
    let p = &menus.play.slots.records[(o / 4) as usize];
    if p.get().name_player().is_none() {
        return Err(BadInput("a row's player without its name").into());
    }
    Ok(p)
}

/// The row's points: the sum of its player's figures.
fn points_at(results: &Results, row: usize) -> Result<u32, Refused> {
    Ok(results.player(offset(results, row) as usize / 4)?.sum)
}

/// The section's first record and the stage's (by `EventState.stage.group` and
/// `EventState.stage.sub`); none: 0.
fn records(
    screens: &Screens,
    stages: &crate::records::stages_store::StageTables,
) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = stages.group_event_table(
        screens.event.read().stage.group,
        screens.event.read().stage.sub,
    )?;
    Ok((
        base,
        crate::records::stages_store::row(stages, base, screens.event.read().stage.number)?,
    ))
}

/// The rows' players, the section's first record, the stage line and the speeds.
struct Prepared<'a> {
    players: Vec<crate::menu::standings::PlayerCell<'a>>,
    base: RecordTable,
    stage: Vec<u8>,
    speeds: Vec<Vec<u8>>,
}

/// What the page needs, checked before anything is written: the rows' players (and points),
/// the stage line, the speeds (3 .. 0x80 characters for `draw_spaced_number`).
fn prepare<'a>(
    o: crate::owners::Owners<'static>,
    menus: &'a Menus,
    results: &Results,
    layout: &Layout,
    stages: &crate::records::stages_store::StageTables,
) -> Result<Prepared<'a>, Refused> {
    let n = rows(menus);
    let players = (0..n)
        .map(|row| player(menus, results, row))
        .collect::<Result<Vec<_>, _>>()?;
    if layout.points {
        for row in 0..n {
            points_at(results, row)?;
        }
    }
    let (base, record) = records(&menus.screens, stages)?;
    let stage = crate::screens::page_settings::stage_line_bytes(
        crate::strings::append_string_by_id::StringOwners::of(o),
        0x482,
        record,
    )?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    let units = crate::strings::units::units(o.strings);
    let mut speeds = Vec::new();
    for p in &players {
        let s = crate::strings::append_speed::append_speed(
            crate::strings::append_string_by_id::StringOwners::of(o),
            p.get().figure,
            false,
            units,
        )?;
        if !(3..=0x80).contains(&s.len()) {
            return Err(BadInput("a speed draw_spaced_number refuses").into());
        }
        speeds.push(s);
    }
    Ok(Prepared {
        players,
        base,
        stage,
        speeds,
    })
}

/// The page. The logic is in `rc2k_core::screens::players_results` (what it needs read and checked
/// first, before anything is drawn).
pub fn draw_players_results(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    results: &Results,
    layout: &Layout,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    use rc2k_core::screens::players_results::{Page, Row, players_results};
    {
        let Prepared {
            players,
            base,
            stage,
            speeds,
        } = prepare(o, menus, results, layout, &system.records().stage_tables)?;
        let mut rows = Vec::new();
        for (row, (p, speed)) in players.iter().zip(speeds).enumerate() {
            rows.push(Row {
                name: crate::screens::game_screen::player_name_of(
                    crate::menu::player_names::name_of(&p.get())?,
                ),
                nation: p.get().nationality,
                car: p.get().car_number,
                time: p.get().time,
                speed,
                points: if layout.points {
                    points_at(results, row)?
                } else {
                    0
                },
            });
        }
        let page = Page {
            section: base.title(&system.records().stage_tables)?,
            stage,
            units_set: crate::strings::units::units(o.strings) != 0,
            show_hours: crate::strings::append_time::show_hours(o.strings),
        };
        players_results(
            &mut crate::screens::game_screen::GameScreen::<
                _,
                _,
                crate::screens::game_screen::HeldText,
            >::new(canvas, o),
            layout,
            &page,
            &rows,
        )
    }
}
