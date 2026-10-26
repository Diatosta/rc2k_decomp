//! The players' stage results pages (`sub_4836CF` with the points column, `sub_48D32E`
//! without; stream A, part F1, 2026-10-18). Both draw the same columns, one loop over the rows
//! each (rows in the order `dword_758720` .. `dword_75872C`, byte offsets into `g_players`,
//! 0x17 apart from y 0xFE), shifted 0x1E right on `sub_48D32E`'s page; their differences are
//! the `Layout`. The original's slips are kept (bugs.md): `sub_4836CF` draws four rows for two
//! players, tests the first row's time with `jl` (the others with equality) and puts the third
//! row's time at 0x1B2.

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::results_store::RESULT_FIGURES;
use crate::text::draw_text::Canvas;

pub use rc2k_core::screens::players_results::{Layout, WITH_POINTS, WITHOUT_POINTS};

fn get(a: u32) -> Result<u32, crate::errors::BadInput> {
    crate::pointers::GamePtr::new(a).read::<u32>("a game value")
}

fn rows(layout: &Layout) -> usize {
    rc2k_core::screens::players_results::rows(layout, EVENT_STATE.player_count.get())
}

fn offset(row: usize) -> u32 {
    RESULT_FIGURES.order[row].get()
}

/// The row's player record, refused for an offset past the four or a missing record or name.
fn player(row: usize) -> Result<u32, Refused> {
    let o = offset(row);
    if o >= 16 || !o.is_multiple_of(4) {
        return Err(BadInput("a row's offset past g_players' four").into());
    }
    let p = get(PLAYER_TABLE.pointers[0].addr() as u32 + o)?;
    if p == 0 || get(p + 0x18)? == 0 {
        return Err(BadInput("a row's player without its record or name").into());
    }
    Ok(p)
}

/// The row's points' place (`off_758992[offset]`).
fn points_at(row: usize) -> Result<u32, Refused> {
    let p = get(CAPTIONS.sum_pointers[0].addr() as u32 + offset(row))?;
    if p == 0 || p == u32::MAX {
        return Err(BadInput("a row without its points").into());
    }
    Ok(p)
}

/// The section's first record and the stage's (`off_735512[dword_7558C8][dword_7558D0]`, +
/// 0xA4 x `dword_7558CC`); none: 0.
fn records() -> Result<(u32, u32), BadInput> {
    let table =
        crate::records::stages_store::STAGE_TABLES.group_list(EVENT_STATE.stage_group.get());
    if table == 0 {
        return Ok((0, 0));
    }
    let base = get(table + 4 * EVENT_STATE.stage_sub.get())?;
    Ok((
        base,
        base.wrapping_add(EVENT_STATE.stage_number.get().wrapping_mul(0xA4)),
    ))
}

/// The rows' players, the section's first record, the stage line and the speeds.
struct Prepared {
    players: Vec<u32>,
    base: u32,
    stage: Vec<u8>,
    speeds: Vec<Vec<u8>>,
}

/// What the page needs, checked before anything is written: the rows' players (and points),
/// the stage line, the speeds (3 .. 0x80 characters for `draw_spaced_number`).
fn prepare(layout: &Layout) -> Result<Prepared, Refused> {
    let n = rows(layout);
    let players = (0..n).map(player).collect::<Result<Vec<_>, _>>()?;
    if layout.points {
        for row in 0..n {
            points_at(row)?;
        }
    }
    let (base, record) = records()?;
    if base == 0 {
        return Err(BadInput("no stage records").into());
    }
    let stage = crate::screens::page_settings::stage_line_bytes(0x482, record)?;
    if stage.len() > 0x100 {
        return Err(BadInput("a stage line past the line buffer").into());
    }
    let units = crate::strings::units::units();
    let mut speeds = Vec::new();
    for p in &players {
        let s = crate::strings::append_speed::append_speed(get(p + 0x94)?, false, units)?;
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

/// The page. Converted: `rc2k_core::screens::players_results` (what it needs read and checked
/// first, before anything is drawn).
pub fn draw_players_results(layout: &Layout, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use rc2k_core::screens::players_results::{Page, Row, players_results};
    {
        let Prepared {
            players,
            base,
            stage,
            speeds,
        } = prepare(layout)?;
        let mut rows = Vec::new();
        for (row, (p, speed)) in players.iter().zip(speeds).enumerate() {
            rows.push(Row {
                name: get(p + 0x18)?,
                nation: get(p + 4)?,
                car: get(p + 8)?,
                time: get(p + 0x88)?,
                speed,
                points: if layout.points {
                    get(points_at(row)?)?
                } else {
                    0
                },
            });
        }
        let page = Page {
            section: get(base + 0x38)?,
            stage,
            units_set: crate::strings::units::units() != 0,
            show_hours: crate::strings::append_time::show_hours(),
            extra_glyph: DRAW_WORK.work_b4.get(),
        };
        players_results(
            &mut crate::screens::game_screen::GameScreen::new(canvas),
            layout,
            &page,
            &rows,
        )
    }
}
