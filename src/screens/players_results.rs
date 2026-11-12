//! The players' stage results pages (`screens::draw_players_results_points` with the points column,
//! `screens::draw_players_results` without). Both draw the same
//! columns, one loop over the rows each (rows in the order `results.order` .. the fourth ranked offset, byte
//! offsets into `g_players`,
//! 0x17 apart from y 0xFE), shifted 0x1E right on `screens::draw_players_results`'s page; their
//! differences are
//! the `Layout`. The original's slips are kept (bugs.md): `screens::draw_players_results_points`
//! draws four rows for two players, tests the first row's time with `jl` (the others with equality)
//! and puts the third row's time at 0x1B2.

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CaptionTables;
use crate::menu::owner::Menus;
use crate::menu::owner::Play;
use crate::menu::owner::Screens;
use crate::records::stages_store::{RecordRef, RecordTable};
use crate::session::DwordLike;
use crate::text::draw_text::Canvas;

pub use rc2k_core::screens::players_results::{Layout, WITH_POINTS, WITHOUT_POINTS};

fn rows(menus: &Menus, layout: &Layout) -> usize {
    rc2k_core::screens::players_results::rows(layout, menus.screens.event.player_count.get())
}

fn offset(play: &Play, row: usize) -> u32 {
    play.results.order[row].get()
}

/// The row's player record, refused for an offset past the four or a missing record or name.
fn player(menus: &Menus, row: usize) -> Result<crate::menu::standings::PlayerCell<'_>, Refused> {
    let play = &menus.play;
    let o = offset(play, row);
    if o >= 16 || !o.is_multiple_of(4) {
        return Err(BadInput("a row's offset past g_players' four").into());
    }
    let p = &play.players.players[(o / 4) as usize];
    if p.get().name_player().is_none() {
        return Err(BadInput("a row's player without its name").into());
    }
    Ok(p)
}

/// The row's points (`CaptionTables.sum_pointers[0][offset]`).
fn points_at<'a>(menus: &'a Menus, play: &Play, row: usize) -> Result<&'a dyn DwordLike, Refused> {
    Ok(CaptionTables::sum_of(
        menus,
        offset(play, row) as usize / 4,
    )?)
}

/// The section's first record and the stage's (`off_735512[Event.stage_group][Event.stage_sub]`, +
/// 0xA4 x `dword_7558CC`); none: 0.
fn records(
    screens: &Screens,
    stages: &crate::records::stages_store::StageTables,
) -> Result<(RecordTable, RecordRef), BadInput> {
    let base = stages.group_event_table(
        screens.event.stage_group.get(),
        screens.event.stage_sub.get(),
    )?;
    Ok((
        base,
        crate::records::stages_store::row(base, screens.event.stage_number.get())?,
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
    menus: &'a Menus,
    layout: &Layout,
    stages: &crate::records::stages_store::StageTables,
) -> Result<Prepared<'a>, Refused> {
    let n = rows(menus, layout);
    let players = (0..n)
        .map(|row| player(menus, row))
        .collect::<Result<Vec<_>, _>>()?;
    if layout.points {
        for row in 0..n {
            points_at(menus, &menus.play, row)?;
        }
    }
    let (base, record) = records(&menus.screens, stages)?;
    let stage = crate::screens::page_settings::stage_line_bytes(0x482, record)?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    let units = crate::strings::units::units();
    let mut speeds = Vec::new();
    for p in &players {
        let s = crate::strings::append_speed::append_speed(p.get().figure, false, units)?;
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
    menus: &Menus,
    system: &crate::system::owner::System,
    layout: &Layout,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    use rc2k_core::screens::players_results::{Page, Row, players_results};
    {
        let Prepared {
            players,
            base,
            stage,
            speeds,
        } = prepare(menus, layout, &system.records.stage_tables)?;
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
                    points_at(menus, &menus.play, row)?.get()
                } else {
                    0
                },
            });
        }
        let page = Page {
            section: base.title()?,
            stage,
            units_set: crate::strings::units::units() != 0,
            show_hours: crate::strings::append_time::show_hours(),
            extra_glyph: menus.screens.draw_work.scratch_b4.get(),
        };
        players_results(
            &mut crate::screens::game_screen::GameScreen::<
                _,
                _,
                crate::screens::game_screen::HeldText,
            >::new(canvas),
            layout,
            &page,
            &rows,
        )
    }
}
